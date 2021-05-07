extern crate chrono;
extern crate near_sdk;

use chrono::Utc;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, PanicOnDefault};
// Implement Moloch Contract

const MAX_VOTING_PERIOD_LENGTH: u128 = 10000000000000000000; // maximum length of voting period;
const MAX_GRACE_PERIOD_LENGTH: u128 = 10000000000000000000; // maximum length of grace period
const MAX_DILUTION_BOUND: u128 = 10000000000000000000; // maximum dilution bound
const MAX_TOKEN_WHITELIST_COUNT: usize = 400; // maximum number of whitelisted tokens

setup_alloc!();
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Moloch {
    period_duration: u128,
    voting_period_length: u128,
    grace_period_length: u128,
    proposal_deposit: u128,
    dilution_bound: u128,
    processing_reward: u128,
    sumononing_time: i64,
    token_whitelist: UnorderedMap<AccountId, bool>,
    deposit_token: AccountId,
    members: UnorderedMap<AccountId, Member>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Member {
    delegate_key: AccountId,
    shares: u128,
    loot: u128,
    exists: bool,
    highest_index_yes_vote: u128,
    jailed: u128,
}

// Needs to be changed to an AccountId
pub type TokenId = u64;

#[derive(Debug, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Null,
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
        approved_tokens: Vec<AccountId>,
        period_duration: u128,
        voting_period_length: u128,
        grace_period_length: u128,
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
        assert!(dilution_bound > 0, "dilution_bound cannot be 0");
        assert!(
            dilution_bound <= MAX_DILUTION_BOUND,
            "dilution_bound exceeds max dilution bound"
        );
        assert!(
            approved_tokens.len() > 0,
            "There needs to be at least one approved token"
        );
        assert!(
            approved_tokens.len() <= MAX_TOKEN_WHITELIST_COUNT,
            "Too many whitelisted tokens"
        );
        assert!(
            proposal_deposit >= processing_reward,
            "proposal_deposit cannot be smaller than processing reward"
        );

        // set deposit token

        // Declare token whitelist mapping
        let mut token_whitelist = UnorderedMap::new(b"token-whitelist".to_vec());
        // Loop over approved tokens
        for token in &approved_tokens {
            assert!(
                env::is_valid_account_id(token.as_bytes()),
                "Token account must be valid"
            );
            println!("{:?}", token_whitelist.get(token));
            assert!(
                token_whitelist.get(&token) == None,
                "Duplicate approved token"
            );
            token_whitelist.insert(&token, &true);
        }

        // Set deposit token
        // TODO: Is this the best way taking the first from an array
        let deposit_token = approved_tokens.get(0).unwrap();

        // Add summoning time
        // Add Member to map
        // TODO: Add Delegate key map, going to omit now because it does not seem necessary
        // Moloch settings
        let mut members = UnorderedMap::new(b"members".to_vec());
        members.insert(
            &summoner,
            &Member {
                delegate_key: summoner.clone(),
                shares: 1,
                loot: 0,
                exists: true,
                highest_index_yes_vote: 0,
                jailed: 0,
            },
        );

        Self {
            period_duration: period_duration,
            voting_period_length: voting_period_length,
            grace_period_length: grace_period_length,
            proposal_deposit: proposal_deposit,
            dilution_bound: dilution_bound,
            processing_reward: processing_reward,
            token_whitelist: token_whitelist,
            sumononing_time: Utc::now().timestamp(),
            deposit_token: deposit_token.to_string(),
            members: members,
        }
    }
    pub fn submit_proposal(
        &self,
        applicant: AccountId,
        shares_requested: u128,
        loot_requested: u128,
        tribute_offered: u128,
        tribute_token: TokenId,
        payment_requested: u128,
        payment_token: TokenId,
        details: String,
    ) -> u128 {
        0
    }
    pub fn submit_whitelist_proposal(
        &self,
        token_to_whitelist: AccountId,
        details: String,
    ) -> u128 {
        0
    }
    pub fn submit_guild_kick_proposal(&self, member_to_kick: AccountId, details: String) -> u128 {
        0
    }
    pub fn submit_vote(&self, proposal_index: u128, uint_vote: u8) {}
    pub fn sponsor_proposal(&self, proposal_id: u128) {}
    pub fn process_proposal(&self, proposal_index: u128) {}
    pub fn process_whitelist_proposal(&self, proposal_index: u128) {}
    pub fn process_guild_kick_proposal(&self, proposal_index: u128) {}
    pub fn rage_quit(&self, shares_to_burn: u128, loot_to_burn: u128) {}
    pub fn rage_kick(&self, acount_id: AccountId) {}
    pub fn withdraw_balance(&self, token: TokenId, amount: u128) {}
    pub fn withdraw_balances(&self, tokens: Vec<AccountId>, amounts: Vec<u128>, max: bool) {}
    pub fn collect_tokens(&self, token: TokenId) {}
    pub fn cancel_proposal(&self, proposal_id: u128) {}
    pub fn update_delegate_key(&self, new_delegate: AccountId) {}
    pub fn can_rage_quit(&self, highest_index_yes_vote: u128) -> bool {
        false
    }
    pub fn has_voting_period_expired(&self, starting_period: u128) -> bool {
        false
    }

    // Getter functions
    pub fn get_current_period(&self) -> u128 {
        0
    }

    pub fn get_proposal_queue_length(&self) -> u128 {
        0
    }

    pub fn get_proposal_flags(&self, proposal_id: u128) -> bool {
        false
    }
    pub fn get_user_token_balance(&self, user: AccountId, token: AccountId) -> u128 {
        0
    }

    pub fn get_member_proposal_vote(
        &self,
        member_address: AccountId,
        proposal_index: u128,
    ) -> Vote {
        Vote::No
    }
    pub fn get_token_count(&self) -> u128 {
        return 0;
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    use std::convert::TryInto;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("bob_near".try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    fn robert() -> AccountId {
        "robert.testnet".to_string()
    }
    fn fdai() -> AccountId {
        "fdai.testnet".to_string()
    }
    #[test]
    fn submit_whitelist_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let proposal = contract.submit_whitelist_proposal("".to_string(), "".to_string());
        assert_eq!(proposal, 0)
    }

    #[test]
    fn submit_guild_kick_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let proposal = contract.submit_guild_kick_proposal("".to_string(), "".to_string());
        assert_eq!(proposal, 0)
    }

    #[test]
    fn sponsor_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.sponsor_proposal(0);
    }
    #[test]
    fn submit_vote() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.submit_vote(0, 0);
    }

    #[test]
    fn process_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.process_proposal(0);
    }

    #[test]
    fn process_whitelist_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.process_whitelist_proposal(0);
    }

    #[test]
    fn process_guild_kick_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.process_guild_kick_proposal(0);
    }

    #[test]
    fn rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.rage_quit(0, 0);
    }

    #[test]
    fn rage_kick() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.rage_kick("".to_string());
    }

    #[test]
    fn withdraw_balance() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.withdraw_balance(0, 0);
    }

    #[test]
    fn withdraw_balances() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.withdraw_balances(vec!["".to_string()], vec![0], false);
    }

    #[test]
    fn collect_tokens() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.collect_tokens(0);
    }

    #[test]
    fn cancel_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.cancel_proposal(0);
    }

    #[test]
    fn update_delegate_key() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        contract.update_delegate_key("".to_string());
    }
    #[test]
    fn can_rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let can = contract.can_rage_quit(0);
        assert_eq!(can, false)
    }

    #[test]
    fn has_voting_period_expired() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let expired = contract.has_voting_period_expired(0);
        assert_eq!(expired, false)
    }

    // Getter Funcitons
    #[test]
    fn get_current_period() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let period = contract.get_current_period();
        assert_eq!(period, 0)
    }

    #[test]
    fn get_proposal_queue_length() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let period = contract.get_proposal_queue_length();
        assert_eq!(period, 0)
    }

    #[test]
    fn get_proposal_flags() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let flags = contract.get_proposal_flags(0);
        assert_eq!(flags, false)
    }

    #[test]
    fn get_user_token_balance() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let balance = contract.get_user_token_balance("".to_string(), "".to_string());
        assert_eq!(balance, 0)
    }

    #[test]
    fn get_member_proposal_vote() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let vote = contract.get_member_proposal_vote("".to_string(), 0);
        assert_eq!(vote, Vote::No)
    }

    #[test]
    fn get_token_count() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::new(robert(), vec![fdai()], 10, 10, 10, 10, 100, 10);
        let count = contract.get_token_count();
        assert_eq!(count, 0)
    }
}
