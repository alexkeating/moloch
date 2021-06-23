extern crate near_contract_standards;
extern crate near_sdk;
extern crate serde;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, PanicOnDefault};

use serde::Serialize;

use std::cmp::max;
use std::collections::HashMap;

mod guild_bank;

// Implement Moloch Contract

const MAX_VOTING_PERIOD_LENGTH: u128 = 10000000000000000000; // maximum length of voting period;
const MAX_GRACE_PERIOD_LENGTH: u128 = 10000000000000000000; // maximum length of grace period
const MAX_DILUTION_BOUND: u128 = 10000000000000000000; // maximum dilution bound

setup_alloc!();
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Moloch {
    period_duration: u128,
    voting_period_length: u128,
    grace_period_length: u128,
    proposal_deposit: u128,
    abort_window: u128,
    dilution_bound: u128,
    processing_reward: u128,
    sumononing_time: u64,
    token_id: AccountId,
    members: UnorderedMap<AccountId, Member>,
    members_by_delegate_key: UnorderedMap<AccountId, AccountId>,
    total_shares: u128,
    bank: guild_bank::GuildBank,
    total_shares_requested: u128,
    proposal_queue: Vector<Proposal>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, PartialEq)]
pub struct Member {
    delegate_key: AccountId,
    shares: u128,
    exists: bool,
    highest_index_yes_vote: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct Proposal {
    proposer: AccountId,
    applicant: AccountId,
    shares_requested: u128,
    starting_period: u128,
    yes_votes: u128,
    no_votes: u128,
    processed: bool,
    did_pass: bool,
    aborted: bool,
    token_tribute: u128,
    details: String,
    max_total_shares_at_yes_vote: u128,
    votes_by_member: HashMap<AccountId, Vote>,
}

// Needs to be changed to an AccountId
pub type TokenId = u64;

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Copy, Clone)]
pub enum Vote {
    Yes,
    No,
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

// Add constructor from the NFT example
// Then start implmenting each function and
// modifing the test
//
// NFT example also has good examples of modifier uses
#[near_bindgen]
impl Moloch {
    #[init]
    pub fn new(
        summoner: AccountId,
        approved_token: AccountId,
        period_duration: u128,
        voting_period_length: u128,
        grace_period_length: u128,
        abort_window: u128,
        proposal_deposit: u128,
        dilution_bound: u128,
        processing_reward: u128,
    ) -> Self {
        // Validate passed in params
        // Log Summon complete
        // Add approved token to whitelist
        // Set Global values
        // Add summoner to Member

        assert!(
            env::is_valid_account_id(summoner.as_bytes()),
            "Summoner must be a valid account"
        );
        assert!(
            env::is_valid_account_id(approved_token.as_bytes()),
            "Approved token must have a valid address"
        );

        assert!(
            period_duration > 0,
            "period_duration must be greater than 0"
        );
        assert!(
            voting_period_length > 0,
            "poting_period length must be greater than 0"
        );
        assert!(
            voting_period_length <= MAX_VOTING_PERIOD_LENGTH,
            "voting_period length must be less than the max voting period"
        );
        assert!(
            grace_period_length <= MAX_GRACE_PERIOD_LENGTH,
            "grace_period exceeds max grace period"
        );
        assert!(abort_window > 0, "Abort window cannot be 0");
        assert!(
            abort_window <= voting_period_length,
            "abort_window must be smaller than the voting_period_length"
        );
        assert!(dilution_bound > 0, "dilution_bound cannot be 0");
        assert!(
            dilution_bound <= MAX_DILUTION_BOUND,
            "dilution_bound exceeds max dilution bound"
        );
        assert!(
            proposal_deposit >= processing_reward,
            "proposal_deposit cannot be smaller than processing reward"
        );

        // create guild bank
        let bank = guild_bank::GuildBank::new(approved_token.clone());

        // TODO: Add Delegate key map, going to omit now because it does not seem necessary
        // Moloch settings
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
            period_duration: period_duration,
            voting_period_length: voting_period_length,
            grace_period_length: grace_period_length,
            proposal_deposit: proposal_deposit,
            abort_window: abort_window,
            dilution_bound: dilution_bound,
            processing_reward: processing_reward,
            token_id: approved_token,
            sumononing_time: env::block_timestamp(),
            members: members,
            members_by_delegate_key: members_by_delegate_key,
            total_shares: 1,
            bank: bank,
            total_shares_requested: 0,
            proposal_queue: Vector::new(b"proposal_queue".to_vec()),
        }
    }
    #[payable]
    pub fn submit_proposal(
        &mut self,
        applicant: AccountId,
        token_tribute: u128,
        shares_requested: u128,
        details: String,
    ) {
        // 0. delegate check
        self.only_delegate();
        // 1. A couple logic checks
        assert!(
            env::is_valid_account_id(applicant.as_bytes()),
            "applicant must be a valid account id"
        );
        let (shares_with_request, shares_requested_overflow) =
            self.total_shares.overflowing_add(shares_requested);
        assert!(!shares_requested_overflow, "Too many shares were requested");
        let (_, shares_overflow) = shares_with_request.overflowing_add(self.total_shares_requested);
        assert!(!shares_overflow, "Too many shares were requested");

        // 2. Add shares
        self.total_shares_requested = self.total_shares_requested.saturating_add(shares_requested);
        // 3. get delegate key
        // only_delegate checks above
        let member_id = self
            .members_by_delegate_key
            .get(&env::predecessor_account_id())
            .unwrap();
        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            env::current_account_id(),
            U128::from(token_tribute),
            Some("proposal token tribute".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 2,
        );
        // TODO: The applicant should also transfer

        // 4. Calculate starting periond
        // TODO: I don't really understand this step

        let mut period_based_on_queue = 0;
        let queue_len = self.proposal_queue.len();
        if queue_len != 0 {
            period_based_on_queue = match self.proposal_queue.get(queue_len.saturating_sub(1)) {
                Some(proposal) => proposal.starting_period,
                None => 0,
            }
        }
        let starting_period = max(self.get_current_period(), period_based_on_queue);

        // 5. Add to queue
        let proposal = Proposal {
            proposer: member_id,
            applicant: applicant,
            shares_requested: shares_requested,
            starting_period: starting_period,
            yes_votes: 0,
            no_votes: 0,
            processed: false,
            did_pass: false,
            aborted: false,
            token_tribute: token_tribute,
            details: details,
            max_total_shares_at_yes_vote: 0,
            votes_by_member: HashMap::new(),
        };
        self.proposal_queue.push(&proposal);
        let proposal_index = self.proposal_queue.len().saturating_sub(1);

        // 6. Log
        env::log(format!("Proposal submitted! proposal_index: {}, sender: {}, member_address: {}, applicant: {}, token_tribute: {}, shares_requested: {} ", proposal_index, env::predecessor_account_id(), proposal.proposer, proposal.applicant, token_tribute, shares_requested).as_bytes());
    }

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
            "proposal does not exist",
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
            "Voting has not begun"
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

    pub fn process_proposal(&mut self, proposal_index: u64) {
        assert!(
            proposal_index < self.proposal_queue.len(),
            "proposal does not exist",
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
        // This cannot overflow because an overflow was checked upon creation of the proposal
        //let total_shares_requested = self
        //    .total_shares_requested
        //    .saturating_sub(proposal.shares_requested);

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

        if passed == true && !proposal.aborted {
            proposal.did_pass = true;
            let member_exists = match self.members.get(&proposal.applicant) {
                Some(_) => true,
                None => false,
            };
            if member_exists {
                let mut member = self.members.get(&proposal.applicant).unwrap();
                // TODO does this need to be saved back in?
                member.shares = member.shares.saturating_add(proposal.shares_requested);
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
                    member.delegate_key = member_delegate_key;
                };

                // TODO: I don't really get this logic
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

                // let shares = self.total_shares.saturating_add(proposal.shares_requested);
                // TODO Transfer to guild bank
                // Send from this contract to the
                // guild contract
                // let prepaid_gas = env::prepaid_gas();
                // ext_fungible_token::ft_transfer_call(
                //     self.bank.to_string(),
                //     U128::from(proposal.token_tribute),
                //     None,
                //     "proposal token tribute for passed proposal".to_string(),
                //     &self.token_id,
                //     0,
                //     prepaid_gas / 2,
                // );
                // assert!()
                // How to get bank id
            }
        }
        // Another else path with a transfer
        // a bunch more transfers

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
    pub fn abort(&self, proposal_index: u64) {
        // Check if proposal index is within the length
        assert!(
            proposal_index < self.proposal_queue.len(),
            "Proposal does not exist"
        );
        // Get the proposal
        let mut proposal = self.proposal_queue.get(proposal_index).unwrap();
        // Check sender is the applicant
        assert!(
            env::predecessor_account_id() == proposal.applicant,
            "Calling account is not the proposal applicant"
        );
        // Check if abort window has passed
        let current_period = self.get_current_period();
        let abort_window = proposal.starting_period.saturating_add(self.abort_window);
        assert!(current_period < abort_window, "Abort window has passed!");
        // Check if proposal has been aborted
        assert!(!proposal.aborted, "Proposal has already been aborted");
        // Reset proposal params for abort
        let token_tribute = proposal.token_tribute;
        proposal.token_tribute = 0;
        proposal.aborted = true;

        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            proposal.applicant,
            U128::from(token_tribute),
            Some("proposal token tribute returned".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 2,
        );

        // Log abort
        env::log(format!("Proposal was aborted by {}", env::predecessor_account_id(),).as_bytes());
    }
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
    pub fn get_current_period(&self) -> u128 {
        let period_64 = env::block_timestamp().saturating_sub(self.sumononing_time);
        u128::from(period_64).wrapping_div(self.period_duration)
    }

    pub fn get_proposal_queue_length(&self) -> u64 {
        return self.proposal_queue.len();
    }

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

    pub fn has_voting_period_expired(&self, starting_period: u128) -> bool {
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

    // helper function
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
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    use std::convert::TryInto;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(bob().try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn robert() -> AccountId {
        "robert.testnet".to_string()
    }
    fn fdai() -> AccountId {
        "fdai.testnet".to_string()
    }
    #[test]
    fn submit_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
    }

    // TODO: Make these error strings a constant
    #[test]
    #[should_panic(expected = r#"applicant must be a valid account id"#)]
    fn submit_proposal_invalid_account() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal("".to_string(), 10, 10, "".to_string());
    }

    #[test]
    #[should_panic(expected = r#"Account is not a delegate"#)]
    fn submit_proposal_not_delegate() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
    }

    #[test]
    fn submit_vote() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract =
            Moloch::new(bob(), fdai(), 10000000000000000000, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
        contract.submit_vote(0, 1);
    }

    // TODO: add these and for successful have better asserts
    // voting has not begun
    // voting has expired
    // member has already voted

    // #[test]
    // fn process_proposal() {
    //     let context = get_context(false);
    //     testing_env!(context);
    //     let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
    //     contract.submit_proposal(robert(), 10, 10, "".to_string());
    //     contract.process_proposal(0);
    // }

    #[test]
    #[should_panic(expected = r#"Account is not a delegate"#)]
    fn process_proposal_not_delegate() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
        contract.process_proposal(0);
    }

    #[test]
    #[should_panic(expected = r#"Proposal is not ready to be processed"#)]
    fn process_proposal_not_ready_to_be_processed() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(bob(), fdai(), 1000000000000000000, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
        contract.process_proposal(0);
    }

    #[test]
    fn rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
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
        let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.update_delegate_key("soda".to_string());
    }

    // Getter
    #[test]
    fn get_current_period() {
        let context = get_context(false);
        testing_env!(context);
        let contract = Moloch::new(robert(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.get_current_period();
    }

    #[test]
    fn get_proposal_queue_length() {
        let context = get_context(false);
        testing_env!(context);
        let contract = Moloch::new(robert(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        let period = contract.get_proposal_queue_length();
        assert_eq!(period, 0)
    }

    #[test]
    fn can_rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(bob(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        contract.submit_proposal(robert(), 10, 10, "".to_string());
        let can = contract.can_rage_quit(0);
        assert_eq!(can, true)
    }

    #[test]
    fn has_voting_period_expired() {
        let context = get_context(false);
        testing_env!(context);
        let contract = Moloch::new(robert(), fdai(), 10, 10, 10, 10, 100, 10, 10);
        let expired = contract.has_voting_period_expired(0);
        assert_eq!(expired, false)
    }

    #[test]
    fn get_member_proposal_vote() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(bob(), fdai(), 10, 1000000000000000000, 10, 10, 100, 10, 10);
        contract.submit_proposal(bob(), 10, 10, "".to_string());
        contract.submit_vote(0, 1);
        let vote = contract.get_member_proposal_vote(bob(), 0);
        assert_eq!(vote, Vote::Yes)
    }
}
