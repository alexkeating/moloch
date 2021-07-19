extern crate near_contract_standards;
extern crate near_sdk;
extern crate serde;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, PanicOnDefault};

use serde::Serialize;

use std::cmp::max;
use std::collections::HashMap;

mod ft_callbacks;
mod guild_bank;

// Implement Moloch Contract

const MAX_VOTING_PERIOD_LENGTH: u64 = 10000000000000000000; // maximum length of voting period;
const MAX_GRACE_PERIOD_LENGTH: u64 = 10000000000000000000; // maximum length of grace period
const MAX_DILUTION_BOUND: u128 = 10000000000000000000; // maximum dilution bound

setup_alloc!();
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Moloch {
    /// The length of period in non_leap nanoseconds
    period_duration: u64,
    /// The number of periods in to vote on a proposal
    voting_period_length: u64,
    /// The number of periods until a proposal is processed
    grace_period_length: u64,
    /// Deposit needed to submit a proposal to combat spam
    proposal_deposit: u128,
    /// Number of periods to abort submitted proposal
    abort_window: u64,
    /// Maximum multiplier a YES voter will be obligated to pay in case of mass ragequit
    dilution_bound: u128,
    /// Amount to give to whoever processes a proposal
    processing_reward: u128,
    /// time used to determine the current period
    summoning_time: u64,
    /// Approved token to use payment
    token_id: AccountId,
    /// Members in the DAO
    members: UnorderedMap<AccountId, Member>,
    /// Members of the DAO related to their delegate key
    members_by_delegate_key: UnorderedMap<AccountId, AccountId>,
    /// Total shares across all members
    total_shares: u128,
    /// Is this even necessary???
    bank: guild_bank::GuildBank,
    /// Total shares that have been requested in unprocessed proposals
    total_shares_requested: u128,
    /// Array of proposals in the order they were submitted
    proposal_queue: Vector<Proposal>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, PartialEq)]
pub struct Member {
    /// The key responsible for submitting proposals and voting - defaults to accountIdD unless updated
    delegate_key: AccountId,
    /// The number of shares assigned to this member
    shares: u128,
    /// Always true once a member has been created
    exists: bool,
    /// Highest proposal index number on which the member voted yes
    highest_index_yes_vote: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Default, PartialEq, Debug)]
pub struct Proposal {
    /// The member who submitted the proposal
    proposer: AccountId,
    /// The applicant who wishes to become a member - this will be used for withdrawls
    applicant: AccountId,
    /// Whether the applicant has sent a proposals tribute
    applicant_has_tributed: bool,
    /// The number of shares the applicant is requesting
    shares_requested: u128,
    /// The period in which voting can start for this proposal
    starting_period: u64,
    /// The total number of yes votes for this proposal
    yes_votes: u128,
    /// The total number of no voters for this prososal
    no_votes: u128,
    /// true if the proposal has been processed
    processed: bool,
    /// true only if the proposal has passed
    did_pass: bool,
    /// true only if an applicant calls the "abort" function before the end of the voting period
    aborted: bool,
    /// Amount of tokens offered as tribute
    token_tribute: u128,
    /// The proposal details - could be an IPFS hash, plaintext, or JSON
    details: String,
    /// The maximum number of total shares encountered at a yes vote on this proposal
    max_total_shares_at_yes_vote: u128,
    /// Mapping of votes for each member
    votes_by_member: HashMap<AccountId, Vote>,
}

// Needs to be changed to an AccountId
pub type TokenId = u64;

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Copy, Clone)]
pub enum Vote {
    Yes,
    No,
    /// Counts as abstention
    Null,
}

impl Vote {
    fn from_u8(value: u8) -> Vote {
        match value {
            1 => Vote::Yes,
            2 => Vote::No,
            3 => Vote::Null,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[near_bindgen]
impl Moloch {
    #[init]
    pub fn new(
        summoner: AccountId,
        approved_token: AccountId,
        period_duration: U64,
        voting_period_length: U64,
        grace_period_length: U64,
        abort_window: U64,
        proposal_deposit: U128,
        dilution_bound: U128,
        processing_reward: U128,
    ) -> Self {
        let _period_duration = u64::from(period_duration);
        let _voting_period_length = u64::from(voting_period_length);
        let _grace_period_length = u64::from(grace_period_length);
        let _proposal_deposit = u128::from(proposal_deposit);
        let _dilution_bound = u128::from(dilution_bound);
        let _processing_reward = u128::from(processing_reward);
        let _abort_window = u64::from(abort_window);

        assert!(
            env::is_valid_account_id(summoner.as_bytes()),
            "Summoner must be a valid account"
        );
        // TODO: Make sure token is valid FungibleToken
        assert!(
            env::is_valid_account_id(approved_token.as_bytes()),
            "Approved token must have a valid address"
        );

        assert!(
            _period_duration > 0,
            "period_duration must be greater than 0"
        );
        assert!(
            _voting_period_length > 0,
            "poting_period length must be greater than 0"
        );
        assert!(
            _voting_period_length <= MAX_VOTING_PERIOD_LENGTH,
            "voting_period length must be less than the max voting period"
        );
        assert!(
            _grace_period_length <= MAX_GRACE_PERIOD_LENGTH,
            "grace_period exceeds max grace period"
        );
        assert!(_abort_window > 0, "Abort window cannot be 0");
        assert!(
            _abort_window <= _voting_period_length,
            "abort_window must be smaller than the voting_period_length"
        );
        assert!(_dilution_bound > 0, "dilution_bound cannot be 0");
        assert!(
            _dilution_bound <= MAX_DILUTION_BOUND,
            "dilution_bound exceeds max dilution bound"
        );
        assert!(
            _proposal_deposit >= _processing_reward,
            "proposal_deposit cannot be smaller than processing reward"
        );

        // create guild bank
        let bank = guild_bank::GuildBank::new(approved_token.clone());

        let mut members = UnorderedMap::new(b"members".to_vec());
        members.insert(
            &summoner,
            &Member {
                delegate_key: summoner.clone(),
                shares: 1,
                exists: true,
                highest_index_yes_vote: 0,
            },
        );

        let mut members_by_delegate_key = UnorderedMap::new(b"members_by_delegate_key".to_vec());
        members_by_delegate_key.insert(&summoner, &summoner);

        // log summon
        env::log(format!("Summon complete by {} with 1 share!", summoner).as_bytes());

        Self {
            period_duration: _period_duration,
            voting_period_length: _voting_period_length,
            grace_period_length: _grace_period_length,
            proposal_deposit: _proposal_deposit,
            abort_window: _abort_window,
            dilution_bound: _dilution_bound,
            processing_reward: _processing_reward,
            token_id: approved_token,
            summoning_time: env::block_timestamp(),
            members: members,
            members_by_delegate_key: members_by_delegate_key,
            total_shares: 1,
            bank: bank,
            total_shares_requested: 0,
            proposal_queue: Vector::new(b"proposal_queue".to_vec()),
        }
    }

    /// At any time members can submit a new proposal using their delegate_key
    ///
    /// 1. This function will update the total requested shares with requested shares \
    /// from this proposal.
    /// 2. It will also transfer the proposal deposit to escrow until the proposal vote is
    /// completed and processed
    /// 3. Calculates the proposal starting period, creates a new proposal and adds it to the
    /// proposal_queue.
    ///
    /// If there are no proposals in the queue or if all the proposals in the queue have already
    /// started their respective voting period, then the proposal start_period will be set to the
    /// next period. If there are proposals in the queue that have not started their voting
    /// period, yet the starting period for the submitted proposal will be the next period after
    /// the last proposal in the queue.
    ///
    /// Existing members can earn additional voting shares through new proposals if they are listed
    /// as the applicant.
    #[payable]
    pub fn submit_proposal(
        &mut self,
        applicant: AccountId,
        token_tribute: U128,
        shares_requested: U128,
        details: String,
    ) {
        // 0. delegate check
        self.only_delegate();
        let _token_tribute = u128::from(token_tribute);
        let _shares_requested = u128::from(shares_requested);

        // 1. A couple logic checks
        assert!(
            env::is_valid_account_id(applicant.as_bytes()),
            "applicant must be a valid account id"
        );
        let (shares_with_request, shares_requested_overflow) =
            self.total_shares.overflowing_add(_shares_requested);
        assert!(!shares_requested_overflow, "Too many shares were requested");
        let (_, shares_overflow) = shares_with_request.overflowing_add(self.total_shares_requested);
        assert!(
            !shares_overflow,
            "Too many shares were requested: due to outstanding shares requested"
        );

        // 2. Add shares
        self.total_shares_requested = self
            .total_shares_requested
            .saturating_add(_shares_requested);
        // 3. get delegate key
        // only_delegate checks above
        let member_id = self
            .members_by_delegate_key
            .get(&env::predecessor_account_id())
            .unwrap();
        let prepaid_gas = env::prepaid_gas();

        // Helpful logs
        // env::log(
        //     format!(
        //         "predecessor {} {} {} {}",
        //         env::predecessor_account_id(),
        //         env::current_account_id(),
        //         env::signer_id(),
        //         &self.proposal_deposit,
        //         &self.token_id
        //     )
        //     .as_bytes(),
        // );
        // If this fails then the whole transaction is not rolled back
        // There should probably be some then logic
        ext_fungible_token::ft_transfer(
            env::current_account_id(),
            U128::from(self.proposal_deposit),
            Some("proposal token tribute".to_string()),
            &self.token_id,
            1,
            prepaid_gas / 2,
        );

        // 4. Calculate starting period
        let mut period_based_on_queue = 0;
        let queue_len = self.proposal_queue.len();
        if queue_len != 0 {
            period_based_on_queue = match self.proposal_queue.get(queue_len.saturating_sub(1)) {
                Some(proposal) => proposal.starting_period,
                None => 0,
            }
        }
        let starting_period =
            max(self.get_current_period(), period_based_on_queue).saturating_add(1);

        // 5. Add to queue
        let proposal = Proposal {
            proposer: member_id,
            applicant: applicant,
            shares_requested: _shares_requested,
            starting_period: starting_period,
            yes_votes: 0,
            no_votes: 0,
            processed: false,
            did_pass: false,
            aborted: false,
            token_tribute: _token_tribute,
            details: details,
            max_total_shares_at_yes_vote: 0,
            votes_by_member: HashMap::new(),
            applicant_has_tributed: false,
        };
        self.proposal_queue.push(&proposal);
        let proposal_index = self.proposal_queue.len().saturating_sub(1);

        // 6. Log
        env::log(format!("Proposal submitted! proposal_index: {}, sender: {}, member_address: {}, applicant: {}, token_tribute: {}, shares_requested: {}", proposal_index, env::predecessor_account_id(), proposal.proposer, proposal.applicant, _token_tribute, _shares_requested).as_bytes());
    }

    #[payable]
    pub fn send_applicant_tribute(&mut self, proposal_index: U64) {
        let _proposal_index = u64::from(proposal_index);
        // Caller must be an active applicant
        // Get proposal and check that the account id matches the proposal applicant id
        //
        // If active applicant find proposal
        let mut proposal = match self.proposal_queue.get(_proposal_index) {
            Some(proposal) => proposal,
            None => panic!("Proposal index does not exist in proposal queue"),
        };

        let caller = env::predecessor_account_id();
        println!("{}", caller);
        assert!(
            proposal.applicant == caller,
            "Caller is not an applicant of this proposal"
        );
        assert!(proposal.aborted == false, "Proposal has been aborted");
        assert!(proposal.processed == false, "Proposal has been processed");

        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            env::current_account_id(),
            U128::from(proposal.token_tribute),
            Some("proposal token tribute".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 2,
        );

        proposal.applicant_has_tributed = true;
        self.proposal_queue.replace(_proposal_index, &proposal);
    }

    /// While a proposal is in its voting period, members can submit their vote using their
    /// delegate_key.
    ///
    /// This function:
    /// 1. Saves the vote on proposal by member address
    /// 2. Based on the vote, adds the member's voting shares to the proposal yesVotes or noVote
    ///    tallies
    /// 3. If the member voted Yes and this is now the highest index proposal they voted yes on, it
    ///    updates their highest_index_yes_vote
    /// 4. If the member voted Yes and this is now the most total shares that the Guild had during
    ///    any Yes vote, update the proposal max_total_shares_at_yes_vote.
    pub fn submit_vote(&mut self, proposal_index: u64, uint_vote: u8) {
        // 0. delegate check
        self.only_delegate();
        // 1. Get member
        let member_id = self
            .members_by_delegate_key
            .get(&env::predecessor_account_id())
            .unwrap();
        let mut member = self.members.get(&member_id).unwrap();
        // 2. Check that proposal exists and fetch
        assert!(
            proposal_index < self.proposal_queue.len(),
            "Proposal does not exist",
        );
        let mut proposal = match self.proposal_queue.get(proposal_index) {
            Some(proposal) => proposal,
            None => panic!("Proposal index does not exist in the proposal_queue"),
        };

        // 3. Create vote
        assert!(
            uint_vote < 3,
            "uint vote must be less than 3, 1 is yes 2 is no"
        );
        let vote = Vote::from_u8(uint_vote);

        // 4. Add some voting period checks
        assert!(
            self.get_current_period() >= proposal.starting_period,
            "Voting period has not begun"
        );
        assert!(
            !self.has_voting_period_expired(proposal.starting_period),
            "Proposal voting period has expired"
        );

        let already_voted = match proposal.votes_by_member.get(&member_id) {
            Some(_) => true,
            None => false,
        };
        assert!(!already_voted, "Member has already voted");
        assert!(!proposal.aborted, "Proposal has been aborted");

        // 5. Store vote
        proposal.votes_by_member.insert(member_id.clone(), vote);
        // 6. Add vote to count
        match vote {
            Vote::Yes => {
                proposal.yes_votes = proposal.yes_votes.saturating_add(member.shares);
                if proposal_index > member.highest_index_yes_vote {
                    member.highest_index_yes_vote = proposal_index;
                };
                if self.total_shares > proposal.max_total_shares_at_yes_vote {
                    proposal.max_total_shares_at_yes_vote = self.total_shares;
                };
            }
            Vote::No => {
                proposal.no_votes = proposal.no_votes.saturating_add(member.shares);
            }
            Vote::Null => {}
        }
        // replace member
        self.members.insert(&member_id, &member);
        self.proposal_queue.replace(proposal_index, &proposal);
        // 7. Log success
        env::log(
            format!(
                "Submitted vote! proposal_index: {}, sender: {}, delegate_key: {}, uint_vote: P{}",
                proposal_index,
                env::predecessor_account_id(),
                member.delegate_key,
                uint_vote,
            )
            .as_bytes(),
        )
    }

    /// After a proposal has completed its grace period, anyone can call process_proposal to tally
    /// the votes and either accept or reject it. The caller will receive a reward for processing
    /// the proposal.
    ///
    /// 1. Sets proposal.processsed = true to prevent duplicate processing
    /// 2. Update total_shares_requested to no longer have the shares requested in the processed
    ///    proposal
    /// 3. Determine if the proposal passed or failed based on the votes and whether or not the
    ///    dilution bound was exceeded
    /// 4. If the proposal passed
    ///    4.1. If the applicant is an existing member, add the requested shares to their existing
    ///      shares to their existing shares
    ///    4.2. If the applicant is a new member, save their data and set their default delegate_key
    ///      to be the same as their member address
    ///      4.2.1. For new members, if the member address is taken by an existing member's
    ///        delegate_key forcibly reset that member's delegate_key to their member address.
    ///    4.3. Update the total shares
    ///    4.4  Transfer the tribute being held in escrow to the guild bank
    /// 5. Otherwise: return all the tribute being held in escrow to the applicant
    /// 6. Send a processing reward to the address that called this function
    /// 7. Send the proposal deposit minus the processing reward to the proposer
    ///
    /// The dilution_bound is a safety net mechanism designed to prevent a memeber from facing a
    /// potentially unbounded grant obligation if they vote YES on a passing proposal and the vast
    /// majority of the other members ragequit before it is processed. The
    /// proposal.max_total_vote_shares_at_yes_no will be the highest total shares at the time of
    /// the yes vote on the proposal. When the proposal is being processed, if members have have
    /// ragequit and the total shares have dropped by more than the dilution_bound (default=3), the proposal
    /// will fail. This means that members voting yes will only be obligated to contribute at most
    /// 3x what they were willing to contribute their share of the proposal cost, if 2/3 of the
    /// shares ragequit
    pub fn process_proposal(&mut self, proposal_index: u64) {
        assert!(
            proposal_index < self.proposal_queue.len(),
            "Proposal does not exist",
        );
        let mut proposal = match self.proposal_queue.get(proposal_index) {
            Some(proposal) => proposal,
            None => panic!("Proposal index does not exist in the proposal_queue"),
        };
        // Check if current period is valid
        assert!(
            self.get_current_period()
                >= proposal
                    .starting_period
                    .saturating_add(self.voting_period_length)
                    .saturating_add(self.grace_period_length),
            "Proposal is not ready to be processed"
        );
        assert!(
            proposal.processed == false,
            "Proposal has already been processed"
        );
        let mut previous_proposal_processed = true;
        if proposal_index != 0 {
            let previous_proposal = match self.proposal_queue.get(proposal_index.saturating_sub(1))
            {
                Some(proposal) => proposal,
                None => panic!("Proposal index does not exist in the proposal_queue"),
            };
            previous_proposal_processed = previous_proposal.processed;
        }

        assert!(
            proposal_index == 0 || previous_proposal_processed == true,
            "Previous proposal must be processed"
        );

        // Set proposal processed to true
        proposal.processed = true;

        // Calculate total shares requested
        self.total_shares_requested = self
            .total_shares_requested
            .saturating_sub(proposal.shares_requested);

        // Check if proposal passed
        let mut passed = proposal.yes_votes > proposal.no_votes;
        // Fail if dilution exceeeded
        let max_total_shares = match self.total_shares_requested.checked_mul(self.dilution_bound) {
            Some(shares) => shares,
            None => u128::MAX,
        };
        if max_total_shares > proposal.max_total_shares_at_yes_vote {
            passed = false
        };

        if passed == true && !proposal.aborted && proposal.applicant_has_tributed {
            proposal.did_pass = true;
            let member_exists = match self.members.get(&proposal.applicant) {
                Some(_) => true,
                None => false,
            };
            if member_exists {
                let mut member = self.members.get(&proposal.applicant).unwrap();
                // TODO does this need to be saved back in?
                member.shares = member.shares.saturating_add(proposal.shares_requested);
                self.members.insert(&proposal.applicant, &member);
            } else {
                let member_delegate_key =
                    match self.members_by_delegate_key.get(&proposal.applicant) {
                        Some(delegate_key) => delegate_key,
                        None => "".to_string(),
                    };
                let member_exists = match self.members.get(&member_delegate_key) {
                    Some(_) => true,
                    None => false,
                };
                if member_exists {
                    let mut member = self.members.get(&member_delegate_key).unwrap();
                    self.members_by_delegate_key
                        .insert(&member_delegate_key, &member_delegate_key);
                    member.delegate_key = member_delegate_key.to_string();
                    self.members.insert(&member_delegate_key, &member);
                };

                // Use applicant account id as delegate key by default
                self.members.insert(
                    &proposal.applicant,
                    &Member {
                        delegate_key: proposal.applicant.clone(),
                        shares: proposal.shares_requested,
                        exists: true,
                        highest_index_yes_vote: 0,
                    },
                );
                self.members_by_delegate_key
                    .insert(&proposal.applicant, &proposal.applicant);
            }
            self.total_shares = self.total_shares.saturating_add(proposal.shares_requested);
            // TODO: Do these promises need to be one after the other
            // Can I await these
            let prepaid_gas = env::prepaid_gas();
            ext_fungible_token::ft_transfer_call(
                env::current_account_id(),
                U128::from(proposal.token_tribute),
                None,
                "proposal token tribute for passed proposal".to_string(),
                &self.token_id,
                0,
                prepaid_gas / 4,
            );
            // TODO: Orginal contract asserts this is successfull

            // Proposal failed and applicant submitted
        } else if proposal.applicant_has_tributed {
            let prepaid_gas = env::prepaid_gas();
            ext_fungible_token::ft_transfer(
                proposal.applicant.clone(),
                U128::from(proposal.token_tribute),
                Some("return proposal token tribute for failed proposal".to_string()),
                &self.token_id,
                0,
                prepaid_gas / 4,
            );
        }

        // TODO: Are these rolled back if the transaction failed
        // Pay processing reward
        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            env::predecessor_account_id(),
            U128::from(proposal.token_tribute),
            Some("pay out processing reward for processing proposal".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 4,
        );

        // Return proposer deposit
        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            proposal.proposer.clone(),
            U128::from(self.proposal_deposit.saturating_sub(self.processing_reward)),
            Some("return proposal deposit for processed proposal".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 4,
        );

        self.proposal_queue.replace(proposal_index, &proposal);

        // Log processed proposal
        env::log(
            format!(
                "Proposal Processed! proposal_index: {}, proposal_applicant: {}, proposal_proposer: {}, proposal_token_tribute: {}, proposal_shares_requested: {}, passed: {}",
                proposal_index,
                proposal.applicant,
                proposal.proposer,
                proposal.token_tribute,
                proposal.shares_requested,
                proposal.did_pass,
            )
            .as_bytes(),
        )
    }

    /// A member can ragequit at any time, so long as the member has not voted Yes on any proposal
    /// in the voting period or grace period, they can irreversibly destroy some of their shares
    /// and receive a proportional sum of the approved token from the Guild Bank.
    ///
    /// 1. Reduce the member's shares by the shares_to_burn being destroyed
    /// 2. Reduce the total shares by the shares_to_burn
    /// 3. Instruct the guild bank to send the member their proportional amount of the approved
    ///    token
    pub fn rage_quit(&mut self, shares_to_burn: u128) {
        // only_member modifier
        self.only_member();
        // Check insuffcient shares
        let mut member = self.members.get(&env::predecessor_account_id()).unwrap();

        assert!(
            member.shares >= shares_to_burn,
            "Not enough shares to be burned"
        );
        // Check can rage_quit
        let can_rage_quit = self.can_rage_quit(member.highest_index_yes_vote);
        assert!(
            can_rage_quit,
            "Can't rage quit until highest index proposal member voted YES is processed",
        );
        // Burn shares
        member.shares = member.shares.saturating_sub(shares_to_burn);
        self.total_shares = self.total_shares.saturating_sub(shares_to_burn);
        // TODO: withdraw shares to burn
        // log rage_quit
        env::log(
            format!(
                "Rage quit! account: {}, shares_burned: {}",
                env::predecessor_account_id(),
                shares_to_burn,
            )
            .as_bytes(),
        );
    }

    /// To avoid a situation where an applicant does not send their tribute in a
    /// timely manner to the proposal the proposer can abort the proposal in order to not
    /// pay the processing reward
    pub fn abort(&mut self, proposal_index: u64) {
        // Check if proposal index is within the length
        assert!(
            proposal_index < self.proposal_queue.len(),
            "Proposal does not exist"
        );
        // Get the proposal
        let mut proposal = self.proposal_queue.get(proposal_index).unwrap();
        // Check sender is the applicant
        assert!(
            env::predecessor_account_id() == proposal.proposer,
            "Calling account is not the proposal proposer"
        );
        assert!(
            proposal.applicant_has_tributed == false,
            "Proposal already has tribute"
        );

        // Check if abort window has passed
        let current_period = self.get_current_period();
        let abort_window = proposal.starting_period.saturating_add(self.abort_window);
        assert!(current_period < abort_window, "Abort window has passed!");
        // Check if proposal has been aborted
        assert!(!proposal.aborted, "Proposal has already been aborted");
        // Reset proposal params for abort
        proposal.aborted = true;

        // return deposit
        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            proposal.proposer.to_string(),
            U128::from(self.proposal_deposit),
            Some("Return the submitted proposal deposit".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 2,
        );

        self.proposal_queue.replace(proposal_index, &proposal);

        // Log abort
        env::log(format!("Proposal was aborted by {}", env::predecessor_account_id(),).as_bytes());
    }

    /// By default when a member is accepted their delegateKey is set to their member accountId. At
    /// any time, they can change it to be any accountId that is not in use, or back to their
    /// accountId.
    ///
    /// 1. Reset the old delegate_key reference in the members_by_delegate_key mapping
    /// 2. Sets the references for the new delegate_key to the member in the
    ///    members_by_delegate_key mapping.
    /// 3. Updates the member delegate_key
    pub fn update_delegate_key(&mut self, new_delegate_key: AccountId) {
        self.only_member();
        // Delegate key cannot be 0
        assert!(
            env::is_valid_account_id(new_delegate_key.as_bytes()),
            "Delegate key cannot be an empty string"
        );
        let sender = env::predecessor_account_id();
        // Skip checks if the member is setting the delegate key to their member address
        if sender != new_delegate_key {
            let member = match self.members.get(&new_delegate_key) {
                Some(member) => member,
                None => Member::default(),
            };
            assert!(!member.exists, "Can't overwrite an exiting member");
            let delegate_key = match self.members_by_delegate_key.get(&new_delegate_key) {
                Some(delegate_key) => delegate_key,
                None => "".to_string(),
            };
            let member_from_delegate_key = match self.members.get(&delegate_key) {
                Some(member) => member,
                None => Member::default(),
            };
            assert!(
                !member_from_delegate_key.exists,
                "Can't overwrite existing delegate keys"
            );
        };
        // Get the member
        let mut member = self.members.get(&sender).unwrap();
        // Overwrite exiting key
        self.members_by_delegate_key
            .insert(&member.delegate_key, &"".to_string());
        // Add new key with send id
        self.members_by_delegate_key
            .insert(&new_delegate_key, &sender);
        // update delegate key on memeber
        member.delegate_key = new_delegate_key;
        // Log delegate key
        env::log(
            format!(
                "Updated delegate key! sender: {}, new_delegate_key: {}",
                sender, member.delegate_key,
            )
            .as_bytes(),
        );
    }

    // Getter functions

    /// The difference between the block_timestamp and the summoning_time is used to figure out how
    /// many periods have elapsed and thus what the current period is.
    pub fn get_current_period(&self) -> u64 {
        let period_64 = env::block_timestamp().saturating_sub(self.summoning_time);
        period_64.wrapping_div(self.period_duration)
    }

    /// Returns the length of the proposal queue
    pub fn get_proposal_queue_length(&self) -> u64 {
        return self.proposal_queue.len();
    }

    /// Returns true if the highest_index_yes_vote has been processed
    pub fn can_rage_quit(&self, highest_index_yes_vote: u64) -> bool {
        assert!(
            highest_index_yes_vote < self.proposal_queue.len(),
            "Proposal does not exist"
        );
        return match self.proposal_queue.get(highest_index_yes_vote) {
            Some(_) => true,
            None => false,
        };
    }

    pub fn has_voting_period_expired(&self, starting_period: u64) -> bool {
        return self.get_current_period()
            >= starting_period.saturating_add(self.voting_period_length);
    }

    pub fn get_member_proposal_vote(&self, member_id: AccountId, proposal_index: u64) -> Vote {
        let member = match self.members.get(&member_id) {
            Some(member) => member,
            None => Member::default(),
        };
        assert!(member.exists, "Member does not exist");
        assert!(proposal_index < self.proposal_queue.len());
        let proposal = self.proposal_queue.get(proposal_index).unwrap();
        return match proposal.votes_by_member.get(&member_id) {
            Some(vote) => *vote,
            None => Vote::Null,
        };
    }

    /// Checks that previous caller is the delegateKey of a
    /// member with at least 1 share
    fn only_delegate(&self) {
        let delegate_key = match self
            .members_by_delegate_key
            .get(&env::predecessor_account_id())
        {
            Some(delegate_key) => delegate_key,
            None => "".to_string(),
        };
        assert!(delegate_key != "".to_string(), "Account is not a delegate");
    }

    /// Checks that the calling account is the address of a member with at least 1 share
    fn only_member(&self) {
        let member = match self.members.get(&env::predecessor_account_id()) {
            Some(member) => member,
            None => Member::default(),
        };
        assert!(member != Member::default(), "Account is not a member");
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, Balance, MockedBlockchain, VMContext};
    use std::convert::TryInto;

    struct MockMember {
        delegate_key: AccountId,
        shares: u128,
        exists: bool,
        highest_index_yes_vote: u64,
    }

    impl MockMember {
        fn new() -> Self {
            MockMember {
                delegate_key: robert(),
                shares: 10,
                exists: true,
                highest_index_yes_vote: 0,
            }
        }

        fn shares(&mut self, shares: u128) -> &mut Self {
            self.shares = shares;
            self
        }

        fn delegate_key(&mut self, delegate_key: AccountId) -> &mut Self {
            self.delegate_key = delegate_key;
            self
        }

        fn build(&self) -> Member {
            Member {
                delegate_key: self.delegate_key.to_string(),
                shares: self.shares,
                exists: true,
                highest_index_yes_vote: self.highest_index_yes_vote,
            }
        }
    }

    // For better test isolation between function calls
    // Create a Mock Propoposal
    struct MockProposal {
        /// The member who submitted the proposal
        proposer: AccountId,
        /// The applicant who wishes to become a member - this will be used for withdrawls
        applicant: AccountId,
        /// Whether the applicant has sent a proposals tribute
        applicant_has_tributed: bool,
        /// The number of shares the applicant is requesting
        shares_requested: u128,
        /// The period in which voting can start for this proposal
        starting_period: u64,
        /// The total number of yes votes for this proposal
        yes_votes: u128,
        /// The total number of no voters for this prososal
        no_votes: u128,
        /// true if the proposal has been processed
        processed: bool,
        /// true only if the proposal has passed
        did_pass: bool,
        /// true only if an applicant calls the "abort" function before the end of the voting period
        aborted: bool,
        /// Amount of tokens offered as tribute
        token_tribute: u128,
        /// The proposal details - could be an IPFS hash, plaintext, or JSON
        details: String,
        /// The maximum number of total shares encountered at a yes vote on this proposal
        max_total_shares_at_yes_vote: u128,
        /// Mapping of votes for each member
        votes_by_member: HashMap<AccountId, Vote>,
    }

    impl MockProposal {
        fn new() -> Self {
            MockProposal {
                proposer: bob(),
                applicant: robert(),
                shares_requested: 10,
                starting_period: 1,
                yes_votes: 0,
                no_votes: 0,
                processed: false,
                did_pass: false,
                aborted: false,
                token_tribute: 12,
                details: "".to_string(),
                max_total_shares_at_yes_vote: 0,
                votes_by_member: HashMap::new(),
                applicant_has_tributed: false,
            }
        }

        fn processed(&mut self, processed: bool) -> &mut Self {
            self.processed = processed;
            self
        }

        fn applicant_has_tributed(&mut self, applicant_has_tributed: bool) -> &mut Self {
            self.applicant_has_tributed = applicant_has_tributed;
            self
        }

        fn aborted(&mut self, aborted: bool) -> &mut Self {
            self.aborted = aborted;
            self
        }

        fn shares_requested(&mut self, shares_requested: u128) -> &mut Self {
            self.shares_requested = shares_requested;
            self
        }

        fn proposer(&mut self, proposer: AccountId) -> &mut Self {
            self.proposer = proposer.to_string();
            self
        }

        fn applicant(&mut self, applicant: AccountId) -> &mut Self {
            self.applicant = applicant.to_string();
            self
        }

        fn yes_vote(&mut self, member: &Member) -> &mut Self {
            self.yes_votes += member.shares;
            self.votes_by_member
                .insert(member.delegate_key.to_string(), Vote::Yes);
            self
        }

        fn no_vote(&mut self, member: &Member) -> &mut Self {
            self.no_votes += member.shares;
            self.votes_by_member
                .insert(member.delegate_key.to_string(), Vote::No);
            self
        }

        // Add to queue
        // Update total_shares_requested
        fn build(&self) -> Proposal {
            let mut votes_by_member = HashMap::new();

            for (key, val) in self.votes_by_member.iter() {
                votes_by_member.insert(key.to_string(), *val);
            }
            Proposal {
                proposer: self.proposer.to_string(),
                applicant: self.applicant.to_string(),
                shares_requested: self.shares_requested,
                starting_period: self.starting_period,
                yes_votes: self.yes_votes,
                no_votes: self.no_votes,
                processed: self.processed,
                did_pass: self.did_pass,
                aborted: self.aborted,
                token_tribute: self.token_tribute,
                details: self.details.to_string(),
                max_total_shares_at_yes_vote: self.max_total_shares_at_yes_vote,
                votes_by_member: votes_by_member,
                applicant_has_tributed: self.applicant_has_tributed,
            }
        }
    }

    struct MockMoloch {
        summoner: AccountId,
        approved_token: AccountId,
        period_duration: U64,
        voting_period_length: U64,
        grace_period_length: U64,
        abort_window: U64,
        proposal_deposit: U128,
        dilution_bound: U128,
        processing_reward: U128,
        proposal_queue: Vector<Proposal>,
        total_shares_requested: u128,
        total_shares: u128,
        members: UnorderedMap<AccountId, Member>,
        members_by_delegate_key: UnorderedMap<AccountId, AccountId>,
    }

    impl MockMoloch {
        fn new() -> Self {
            MockMoloch {
                summoner: bob(),
                approved_token: fdai(),
                // period_duration 30 seconds
                period_duration: 10u64.pow(9).into(),
                voting_period_length: 3.into(),
                grace_period_length: 2.into(),
                abort_window: 1.into(),
                proposal_deposit: 100.into(),
                dilution_bound: 10.into(),
                processing_reward: 10.into(),
                proposal_queue: Vector::new(b"proposal_queue".to_vec()),
                total_shares_requested: 0,
                total_shares: 0,
                members: UnorderedMap::new(b"mock_members".to_vec()),
                members_by_delegate_key: UnorderedMap::new(
                    b"mock_members_by_delegate_key".to_vec(),
                ),
            }
        }

        fn summoner(&mut self, summoner: AccountId) -> &mut Self {
            self.summoner = summoner;
            self
        }

        fn add_proposal(&mut self, proposal: Proposal) -> &mut Self {
            self.proposal_queue.push(&proposal);
            self.total_shares_requested += proposal.shares_requested;
            self
        }

        fn add_member(&mut self, member: Member) -> &mut Self {
            self.members_by_delegate_key
                .insert(&member.delegate_key, &member.delegate_key);
            self.members.insert(&member.delegate_key, &member);
            self.total_shares += member.shares;
            self
        }

        fn update_member_delegate_key(
            &mut self,
            delegate_key: &AccountId,
            account_id: &AccountId,
        ) -> &mut Self {
            self.members_by_delegate_key
                .insert(&delegate_key, &account_id);
            let mut member = self.members.get(&account_id).unwrap();
            member.delegate_key = delegate_key.to_string();
            self.members.insert(&account_id, &member);
            self
        }

        fn build(&self) -> Moloch {
            let mut moloch = Moloch::new(
                self.summoner.to_string(),
                self.approved_token.to_string(),
                self.period_duration,
                self.voting_period_length,
                self.grace_period_length,
                self.abort_window,
                self.proposal_deposit,
                self.dilution_bound,
                self.processing_reward,
            );
            moloch.proposal_queue.extend(self.proposal_queue.iter());
            moloch.total_shares_requested += self.total_shares_requested;
            moloch.total_shares += self.total_shares;
            moloch.members.extend(self.members.iter());
            moloch
                .members_by_delegate_key
                .extend(self.members_by_delegate_key.iter());
            moloch
        }
    }

    // Create a Mock member

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(bob().try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    fn get_context_builder(is_view: bool) -> VMContextBuilder {
        VMContextBuilder::new()
            .signer_account_id(bob().try_into().unwrap())
            .is_view(is_view)
            .clone()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn robert() -> AccountId {
        "robert.testnet".to_string()
    }

    fn alice() -> AccountId {
        "alice.testnet".to_string()
    }

    fn fdai() -> AccountId {
        "fdai.testnet".to_string()
    }

    /// Tests for submit propposal
    #[test]
    fn submit_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal(robert(), 12.into(), 10.into(), "".to_string());

        let proposal = contract.proposal_queue.get(0);
        let expected_proposal = Proposal {
            proposer: bob(),
            applicant: robert(),
            shares_requested: 10,
            starting_period: 1,
            yes_votes: 0,
            no_votes: 0,
            processed: false,
            did_pass: false,
            aborted: false,
            token_tribute: 12,
            details: "".to_string(),
            max_total_shares_at_yes_vote: 0,
            votes_by_member: HashMap::new(),
            applicant_has_tributed: false,
        };
        let logs = get_logs();

        assert_eq!(proposal.unwrap(), expected_proposal);
        assert_eq!(contract.total_shares_requested, 10);
        let log = logs.get(1);
        assert_eq!(*log.unwrap(), format!("Proposal submitted! proposal_index: 0, sender: {}, member_address: {}, applicant: {}, token_tribute: 12, shares_requested: 10", bob().to_string(), bob().to_string(), robert().to_string()));
    }
    // TODO: Integration Check if contract has the proper amount from submitting a
    // proposal

    // Add test with multiple proposals
    #[test]
    fn submit_proposal_multiple_proposals() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal(robert(), 12.into(), 10.into(), "".to_string());

        let context = get_context(false);
        testing_env!(context);
        contract.submit_proposal(robert(), 20.into(), 20.into(), "".to_string());

        let proposal = contract.proposal_queue.get(1);
        let expected_proposal = Proposal {
            proposer: bob(),
            applicant: robert(),
            shares_requested: 20,
            starting_period: 2,
            yes_votes: 0,
            no_votes: 0,
            processed: false,
            did_pass: false,
            aborted: false,
            token_tribute: 20,
            details: "".to_string(),
            max_total_shares_at_yes_vote: 0,
            votes_by_member: HashMap::new(),
            applicant_has_tributed: false,
        };
        assert_eq!(proposal.unwrap(), expected_proposal);
        assert_eq!(contract.total_shares_requested, 30);
    }

    // TODO: Make these error strings a constant
    #[test]
    #[should_panic(expected = r#"applicant must be a valid account id"#)]
    fn submit_proposal_invalid_account() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal("".to_string(), 10.into(), 10.into(), "".to_string());
    }

    #[test]
    #[should_panic(expected = r#"Too many shares were requested"#)]
    fn submit_proposal_shares_requested_overflow() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal(robert(), 10.into(), u128::MAX.into(), "".to_string());
    }

    #[test]
    #[should_panic(
        expected = r#"Too many shares were requested: due to outstanding shares requested"#
    )]
    fn submit_proposal_total_shares_requested_overflow() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal(
            robert(),
            10.into(),
            u128::MAX.saturating_sub(1).into(),
            "".to_string(),
        );
        contract.submit_proposal(robert(), 10.into(), 1.into(), "".to_string());
    }

    #[test]
    #[should_panic(expected = r#"Account is not a delegate"#)]
    fn submit_proposal_not_delegate() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().summoner(robert()).build();
        contract.submit_proposal(robert(), 10.into(), 10.into(), "".to_string());
    }

    #[test]
    fn send_applicant_tribute() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();

        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.applicant_has_tributed, false);

        let context_robert = get_context_builder(false)
            .predecessor_account_id(proposal.applicant.try_into().unwrap())
            .build();

        testing_env!(context_robert);
        contract.send_applicant_tribute(0.into());
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.applicant_has_tributed, true)
    }

    // Proposal does not exist at index
    #[test]
    #[should_panic(expected = r#"Proposal index does not exist in proposal queue"#)]
    fn send_applicant_tribute_no_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();

        let context = get_context(false);
        testing_env!(context);
        contract.send_applicant_tribute(5.into());
    }
    // Caller is not the applicant
    #[test]
    #[should_panic(expected = r#"Caller is not an applicant of this proposal"#)]
    fn send_applicant_tribute_caller_not_applicant() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new()
            .aborted(true)
            .applicant(robert())
            .build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        // contract.submit_proposal(robert(), 10.into(), 10.into(), "".to_string());

        let context = get_context(false);
        testing_env!(context);
        contract.send_applicant_tribute(0.into());
    }

    // Proposal has already been aborted
    #[test]
    #[should_panic(expected = r#"Proposal has been aborted"#)]
    fn send_applicant_tribute_proposal_has_been_aborted() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().aborted(true).build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();

        let proposal = contract.proposal_queue.get(0).unwrap();
        let context = get_context_builder(false)
            .predecessor_account_id(proposal.applicant.try_into().unwrap())
            .build();
        testing_env!(context);
        contract.send_applicant_tribute(0.into());
    }

    // Proposal has already been processed
    #[test]
    #[should_panic(expected = r#"Proposal has been processed"#)]
    fn send_applicant_tribute_proposal_has_been_processed() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new()
            .processed(true)
            .proposer(robert())
            .applicant(bob())
            .build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();

        let context = get_context(false);
        testing_env!(context);
        contract.send_applicant_tribute(0.into());
    }

    // Single vote
    #[test]
    fn submit_vote_yes() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();

        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp((contract.summoning_time + contract.period_duration).into())
            .build();
        testing_env!(context);
        contract.submit_vote(0, 1);

        // Assert that the correct proposal was voted on
        // and that it was marked as a yes
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.yes_votes, 1);
        assert_eq!(proposal.no_votes, 0);
        assert_eq!(proposal.max_total_shares_at_yes_vote, 1);
        let member = contract.members.get(&bob()).unwrap();
        assert_eq!(member.highest_index_yes_vote, 0);
    }

    // TODO: add these and for successful have better asserts
    // Multiple yes and no votes, do in integration/ end to end tests
    // No vote
    #[test]
    fn submit_vote_no() {
        // Setup
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();

        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp((contract.summoning_time + contract.period_duration).into())
            .build();

        testing_env!(context);
        contract.submit_vote(0, 2);

        // Assert that the correct proposal was voted on
        // and that it was marked as a yes
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.yes_votes, 0);
        assert_eq!(proposal.no_votes, 1);
        let member = contract.members.get(&bob()).unwrap();
        assert_eq!(member.highest_index_yes_vote, 0);
    }

    // Two proposals
    // 3 users
    // multiple votes yes and not
    #[test]
    fn submit_vote_complex() {
        // Setup the test contract
        let context = get_context(false);
        testing_env!(context);
        let proposal_one = MockProposal::new().build();
        let proposal_two = MockProposal::new().build();
        let robert_member_info = MockMember::new().delegate_key(robert()).shares(30).build();
        let alice_member_info = MockMember::new().delegate_key(alice()).shares(50).build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal_one)
            .add_proposal(proposal_two)
            .add_member(robert_member_info)
            .add_member(alice_member_info)
            .build();

        // Make sure two periods pass so each proposal can
        // be voted on
        let mut context_builder = get_context_builder(false);
        let context_block_timestamp = context_builder
            .block_timestamp((contract.summoning_time + contract.period_duration * 2).into());
        testing_env!(context_block_timestamp.build());

        // Actions by bob
        contract.submit_vote(0, 1);
        testing_env!(context_block_timestamp.build());
        contract.submit_vote(1, 1);
        let member = contract.members.get(&bob()).unwrap();
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(
            member.highest_index_yes_vote, 1,
            "Highest proposal index is wrong"
        );
        assert_eq!(
            proposal.max_total_shares_at_yes_vote, 81,
            "Max number of total shares is wrong"
        );

        // Roberts actions
        testing_env!(context_block_timestamp
            .predecessor_account_id(robert().try_into().unwrap())
            .build());

        contract.submit_vote(0, 1);
        testing_env!(context_block_timestamp.build());
        contract.submit_vote(1, 2);
        let member = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member.highest_index_yes_vote, 0,
            "Highest proposal index is wrong for robert"
        );

        // Alices actions
        testing_env!(context_block_timestamp
            .predecessor_account_id(alice().try_into().unwrap())
            .build());

        contract.submit_vote(0, 1);
        testing_env!(context_block_timestamp.build());
        contract.submit_vote(1, 1);
        let member = contract.members.get(&alice()).unwrap();
        assert_eq!(
            member.highest_index_yes_vote, 1,
            "Highest proposal index is wrong for alice"
        );

        let proposal_zero = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal_zero.yes_votes, 81);
        assert_eq!(proposal_zero.no_votes, 0);
        assert_eq!(proposal_zero.processed, false);
        assert_eq!(proposal_zero.aborted, false);

        let proposal_one = contract.proposal_queue.get(1).unwrap();
        assert_eq!(proposal_one.yes_votes, 51);
        assert_eq!(proposal_one.no_votes, 30);
        assert_eq!(proposal_one.processed, false);
        assert_eq!(proposal_one.aborted, false);
    }

    // Proposal does not exist
    #[test]
    #[should_panic(expected = r#"Proposal does not exist"#)]
    fn submit_vote_no_existing_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_vote(0, 1)
    }

    #[test]
    #[should_panic(expected = r#"uint vote must be less than 3, 1 is yes 2 is no"#)]
    pub fn submit_vote_undefined_vote() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        contract.submit_vote(0, 3)
    }

    // voting has not begun
    #[test]
    #[should_panic(expected = r#"Voting period has not begun"#)]
    pub fn submit_vote_has_not_begun() {
        let context = get_context(false);
        testing_env!(context);

        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        contract.submit_vote(0, 2)
    }

    // voting has expired
    #[test]
    #[should_panic(expected = r#"Proposal voting period has expired"#)]
    pub fn submit_vote_expired() {
        let context = get_context(false);
        testing_env!(context);

        let mut context_builder = get_context_builder(false);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        let block_time = contract.summoning_time
            + (contract.period_duration * (contract.voting_period_length + 1));
        let context = context_builder.block_timestamp(block_time.into()).build();
        testing_env!(context);

        contract.submit_vote(0, 2)
    }

    // member has already voted
    #[test]
    #[should_panic(expected = r#"Member has already voted"#)]
    pub fn submit_vote_member_already_voted() {
        let context = get_context(false);
        testing_env!(context);

        let mut context_builder = get_context_builder(false);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        let block_time = contract.summoning_time + contract.period_duration;
        let context = context_builder.block_timestamp(block_time.into()).build();
        testing_env!(context);

        contract.submit_vote(0, 2);
        contract.submit_vote(0, 1);
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.no_votes, 1)
    }

    // proposal has been aborted
    #[test]
    #[should_panic(expected = r#"Proposal has been aborted"#)]
    pub fn submit_vote_proposal_aborted() {
        let context = get_context(false);
        testing_env!(context);

        let mut context_builder = get_context_builder(false);

        let proposal = MockProposal::new().aborted(true).build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        let block_time = contract.summoning_time + contract.period_duration;
        let context = context_builder.block_timestamp(block_time.into()).build();
        testing_env!(context);
        contract.submit_vote(0, 2);
    }

    // Member not delegate
    #[test]
    #[should_panic(expected = r#"Account is not a delegate"#)]
    pub fn submit_vote_member_not_delegate() {
        let context = get_context(false);
        testing_env!(context);

        let mut context_builder = get_context_builder(false);

        let proposal = MockProposal::new().aborted(true).build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        let block_time = contract.summoning_time + contract.period_duration;
        let context = context_builder
            .block_timestamp(block_time.into())
            .predecessor_account_id(alice().try_into().unwrap())
            .build();
        testing_env!(context);
        contract.submit_vote(0, 2);
    }

    // Test passed proposal
    #[test]
    fn process_proposal_passed() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().build();
        let proposal = MockProposal::new()
            .yes_vote(&member)
            .applicant_has_tributed(true)
            .build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal)
            .add_member(member)
            .build();
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, false, "Proposal has been processed");
        assert_eq!(
            contract.total_shares_requested, proposal.shares_requested,
            "Total shares requested has not been set correctly",
        );
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        contract.process_proposal(0);

        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, true, "Proposal has not been processed");
        assert_eq!(
            contract.total_shares_requested, 0,
            "Number of requested shares has not been subtracted",
        );
        let member = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member.shares, 20,
            "Member does not have the correct number not shares"
        );
        assert_eq!(
            contract.total_shares, 21,
            "Total shares has not been updated correctly"
        );
    }

    // Test passed proposal existing member, Assert shares are added
    #[test]
    fn process_proposal_passed_existing_member() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().shares(1).delegate_key(bob()).build();
        let proposal = MockProposal::new()
            .applicant(alice())
            .yes_vote(&member)
            .shares_requested(15)
            .applicant_has_tributed(true)
            .build();

        let existing_member = MockMember::new().shares(7).delegate_key(alice()).build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal)
            .add_member(existing_member)
            .build();
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        contract.process_proposal(0);

        let member = contract.members.get(&alice()).unwrap();
        assert_eq!(
            member.shares, 22,
            "Member does not have the correct number not shares"
        );
        assert_eq!(
            contract.total_shares, 23,
            "Total shares has not been updated correctly"
        );
    }

    // Test passed proposal New member, assert member saved,
    #[test]
    fn process_proposal_passed_new_member() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().shares(1).delegate_key(bob()).build();
        let proposal = MockProposal::new()
            .applicant(alice())
            .yes_vote(&member)
            .shares_requested(15)
            .applicant_has_tributed(true)
            .build();

        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        contract.process_proposal(0);

        let member = contract.members.get(&alice()).unwrap();
        let member_delegate_key = contract.members_by_delegate_key.get(&alice()).unwrap();
        assert_eq!(
            member.shares, 15,
            "Member does not have the correct number not shares"
        );
        assert_eq!(
            member.delegate_key,
            alice().to_string(),
            "Member delegate_key is incorrect"
        );
        assert_eq!(member.exists, true, "Member does not exists");
        assert_eq!(
            member.highest_index_yes_vote, 0,
            "Highest index vote is incorrect"
        );
        assert_eq!(member_delegate_key, alice().to_string());
        assert_eq!(
            contract.total_shares, 16,
            "Total shares has not been updated correctly"
        );
    }

    // Test passed proposal New member, delegate_key already exists
    #[test]
    fn process_proposal_passed_new_member_existing_delegate_key() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().shares(10).delegate_key(robert()).build();
        let proposal = MockProposal::new()
            .applicant(alice())
            .yes_vote(&member)
            .shares_requested(15)
            .applicant_has_tributed(true)
            .build();

        let mut contract = MockMoloch::new()
            .add_proposal(proposal)
            .add_member(member)
            .update_member_delegate_key(&alice(), &robert())
            .build();
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        let member_robert = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member_robert.delegate_key,
            alice(),
            "Delegate key has not been setup correctly"
        );
        contract.process_proposal(0);

        let member_robert = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member_robert.delegate_key,
            robert(),
            "Delegate key has not been updated correctly"
        );

        let member = contract.members.get(&alice()).unwrap();
        let member_delegate_key = contract.members_by_delegate_key.get(&alice()).unwrap();
        assert_eq!(
            member.shares, 15,
            "Member does not have the correct number not shares"
        );
        assert_eq!(
            member.delegate_key,
            alice().to_string(),
            "Member delegate_key is incorrect"
        );
        assert_eq!(member.exists, true, "Member does not exists");
        assert_eq!(
            member.highest_index_yes_vote, 0,
            "Highest index vote is incorrect"
        );
        assert_eq!(member_delegate_key, alice().to_string());
        assert_eq!(
            contract.total_shares, 26,
            "Total shares has not been updated correctly"
        );
    }

    // Test cases process proposal
    // Assert proposal has processed set to true
    // Assert total_shares_requested has the proposal shares subtracted
    // Test failed proposal too many nos
    #[test]
    fn process_proposal_failed() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().build();
        let proposal = MockProposal::new()
            .no_vote(&member)
            .applicant_has_tributed(true)
            .build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal)
            .add_member(member)
            .build();
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, false, "Proposal has been processed");
        assert_eq!(
            contract.total_shares_requested, proposal.shares_requested,
            "Total shares requested has not been set correctly",
        );
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        contract.process_proposal(0);

        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, true, "Proposal has not been processed");
        assert_eq!(
            contract.total_shares_requested, 0,
            "Number of requested shares has not been subtracted",
        );
        let member = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member.shares, 10,
            "Member does not have the correct number not shares"
        );
        assert_eq!(contract.total_shares, 11, "Total shares is not correct");
    }

    // Test failed proposal no tribute sent
    #[test]
    fn process_proposal_failed_no_tribute_sent() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().build();
        let proposal = MockProposal::new()
            .yes_vote(&member)
            .applicant_has_tributed(false)
            .build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal)
            .add_member(member)
            .build();
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, false, "Proposal has been processed");
        assert_eq!(
            contract.total_shares_requested, proposal.shares_requested,
            "Total shares requested has not been set correctly",
        );
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        contract.process_proposal(0);

        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, true, "Proposal has not been processed");
        assert_eq!(
            contract.total_shares_requested, 0,
            "Number of requested shares has not been subtracted",
        );
        let member = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member.shares, 10,
            "Member does not have the correct number not shares"
        );
        assert_eq!(contract.total_shares, 11, "Total shares is not correct");
    }

    // Test failed proposal aborted
    #[test]
    fn process_proposal_failed_aborted() {
        let context = get_context(false);
        testing_env!(context);
        let member = MockMember::new().build();
        let proposal = MockProposal::new()
            .yes_vote(&member)
            .applicant_has_tributed(true)
            .aborted(true)
            .build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal)
            .add_member(member)
            .build();
        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, false, "Proposal has been processed");
        assert_eq!(
            contract.total_shares_requested, proposal.shares_requested,
            "Total shares requested has not been set correctly",
        );
        let mut context_builder = get_context_builder(false);
        let context = context_builder
            .block_timestamp(
                contract.summoning_time
                    + (contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)),
            )
            .build();
        testing_env!(context);
        contract.process_proposal(0);

        let proposal = contract.proposal_queue.get(0).unwrap();
        assert_eq!(proposal.processed, true, "Proposal has not been processed");
        assert_eq!(
            contract.total_shares_requested, 0,
            "Number of requested shares has not been subtracted",
        );
        let member = contract.members.get(&robert()).unwrap();
        assert_eq!(
            member.shares, 10,
            "Member does not have the correct number not shares"
        );
        assert_eq!(contract.total_shares, 11, "Total shares is not correct");
    }

    // Proposall does not exist
    #[test]
    #[should_panic(expected = r#"Proposal does not exist"#)]
    fn process_proposal_does_not_exist() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.process_proposal(0);
    }

    #[test]
    #[should_panic(expected = r#"Proposal is not ready to be processed"#)]
    fn process_proposal_not_ready_to_be_processed() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        contract.process_proposal(0);
    }

    #[test]
    #[should_panic(expected = r#"Proposal has already been processed"#)]
    fn process_proposal_already_processed() {
        let context = get_context(false);
        testing_env!(context);
        let proposal = MockProposal::new().processed(true).build();
        let mut contract = MockMoloch::new().add_proposal(proposal).build();
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder
            .block_timestamp(
                contract.summoning_time
                    + contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)
            )
            .build());

        contract.process_proposal(0);
    }

    #[test]
    #[should_panic(expected = r#"Previous proposal must be processed"#)]
    fn process_proposal_previous_proposal_not_processed() {
        let context = get_context(false);
        testing_env!(context);
        let proposal_one = MockProposal::new().build();
        let proposal_two = MockProposal::new().build();
        let mut contract = MockMoloch::new()
            .add_proposal(proposal_one)
            .add_proposal(proposal_two)
            .build();
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder
            .block_timestamp(
                contract.summoning_time
                    + contract.period_duration
                        * (contract.voting_period_length + contract.grace_period_length + 1)
            )
            .build());
        contract.process_proposal(1);
    }

    #[test]
    fn rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal(robert(), 10.into(), 10.into(), "".to_string());
        contract.rage_quit(0);
    }

    // TODO: Figure out how to Mock the moloch
    // to avoid abort window length issues
    // #[test]
    // #[should_panic(expected = r#"Abort window has passed!"#)]
    // fn abort() {
    //     let context = get_context(false);
    //     testing_env!(context);
    //     let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
    //     contract.submit_proposal(bob(), 10, 10, "".to_string());
    //     contract.abort(0);
    // }

    #[test]
    fn update_delegate_key() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.update_delegate_key("soda".to_string());
    }

    // Getter
    #[test]
    fn get_current_period() {
        let context = get_context(false);
        testing_env!(context);
        let contract = MockMoloch::new().build();
        contract.get_current_period();
    }

    #[test]
    fn get_proposal_queue_length() {
        let context = get_context(false);
        testing_env!(context);
        let contract = MockMoloch::new().build();
        let period = contract.get_proposal_queue_length();
        assert_eq!(period, 0)
    }

    #[test]
    fn can_rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = MockMoloch::new().build();
        contract.submit_proposal(robert(), 10.into(), 10.into(), "".to_string());
        let can = contract.can_rage_quit(0);
        assert_eq!(can, true)
    }

    #[test]
    fn has_voting_period_expired() {
        let context = get_context(false);
        testing_env!(context);
        let contract = Moloch::new(
            robert(),
            fdai(),
            10.into(),
            10.into(),
            10.into(),
            10.into(),
            100.into(),
            10.into(),
            10.into(),
        );
        let expired = contract.has_voting_period_expired(0);
        assert_eq!(expired, false)
    }

    // Testing time manipulation
    // #[test]
    // fn get_member_proposal_vote() {
    //     let context = get_context(false);
    //     testing_env!(context);
    //     let mut contract = Moloch::new(bob(), fdai(), 10, 1000000000000000000, 10, 10, 100, 10, 10);
    //     contract.submit_proposal(bob(), 10, 10, "".to_string());
    //     contract.submit_vote(0, 1);
    //     let vote = contract.get_member_proposal_vote(bob(), 1);
    //     assert_eq!(vote, Vote::Yes)
    // }
}
