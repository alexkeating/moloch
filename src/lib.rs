extern crate near_sdk;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, setup_alloc, AccountId};
// Implement Moloch Contract

setup_alloc!();
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Moloch {}

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
    #[test]
    fn submit_whitelist_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let proposal = contract.submit_whitelist_proposal("".to_string(), "".to_string());
        assert_eq!(proposal, 0)
    }

    #[test]
    fn submit_guild_kick_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let proposal = contract.submit_guild_kick_proposal("".to_string(), "".to_string());
        assert_eq!(proposal, 0)
    }

    #[test]
    fn sponsor_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.sponsor_proposal(0);
    }
    #[test]
    fn submit_vote() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.submit_vote(0, 0);
    }

    #[test]
    fn process_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.process_proposal(0);
    }

    #[test]
    fn process_whitelist_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.process_whitelist_proposal(0);
    }

    #[test]
    fn process_guild_kick_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.process_guild_kick_proposal(0);
    }

    #[test]
    fn rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.rage_quit(0, 0);
    }

    #[test]
    fn rage_kick() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.rage_kick("".to_string());
    }

    #[test]
    fn withdraw_balance() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.withdraw_balance(0, 0);
    }

    #[test]
    fn withdraw_balances() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.withdraw_balances(vec!["".to_string()], vec![0], false);
    }

    #[test]
    fn collect_tokens() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.collect_tokens(0);
    }

    #[test]
    fn cancel_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.cancel_proposal(0);
    }

    #[test]
    fn update_delegate_key() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.update_delegate_key("".to_string());
    }
    #[test]
    fn can_rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let can = contract.can_rage_quit(0);
        assert_eq!(can, false)
    }

    #[test]
    fn has_voting_period_expired() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let expired = contract.has_voting_period_expired(0);
        assert_eq!(expired, false)
    }

    // Getter Funcitons
    #[test]
    fn get_current_period() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let period = contract.get_current_period();
        assert_eq!(period, 0)
    }

    #[test]
    fn get_proposal_queue_length() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let period = contract.get_proposal_queue_length();
        assert_eq!(period, 0)
    }

    #[test]
    fn get_proposal_flags() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let flags = contract.get_proposal_flags(0);
        assert_eq!(flags, false)
    }

    #[test]
    fn get_user_token_balance() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let balance = contract.get_user_token_balance("".to_string(), "".to_string());
        assert_eq!(balance, 0)
    }

    #[test]
    fn get_member_proposal_vote() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let vote = contract.get_member_proposal_vote("".to_string(), 0);
        assert_eq!(vote, Vote::No)
    }

    #[test]
    fn get_token_count() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let count = contract.get_token_count();
        assert_eq!(count, 0)
    }
}
