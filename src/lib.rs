extern crate bigint;
extern crate near_sdk;

use bigint::uint::U256;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, setup_alloc, AccountId};
// Implement Moloch Contract

setup_alloc!();
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Moloch {}

pub type TokenId = u64;

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
    pub fn submit_vote(&self, proposalIndex: u128, uintVote: u8) {}
    pub fn process_proposal(&self, proposalIndex: u128) {}
    pub fn rage_quit(&self, shares_to_burn: u128) {}
    pub fn abort(&self, proposal_index: u128) {}
    pub fn update_delegate_key(&self, new_delegate: AccountId) {}
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
    fn submit_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        let proposal = contract.submit_proposal("ID".to_string(), 0, 0, 0, 0, 0, 0, "".to_string());
        assert_eq!(proposal, 0)
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
    fn rage_quit() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.rage_quit(0);
    }

    #[test]
    fn abort() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.abort(0);
    }

    #[test]
    fn update_delegate_key() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Moloch::default();
        contract.update_delegate_key("".to_string());
    }
}
