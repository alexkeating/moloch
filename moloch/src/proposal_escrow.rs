extern crate near_sdk;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ProposalEscrow {
    user_balances: UnorderedMap<AccountId, u128>,
}

// This can only be called internally
// on the token Receiver
impl ProposalEscrow {
    pub fn new() -> Self {
        Self {
            user_balances: UnorderedMap::new(b"user_balances".to_vec()),
        }
    }

    pub fn deposit(&mut self, account_id: AccountId, amount: u128) -> u128 {
        let balance = match self.user_balances.get(&account_id) {
            Some(balance) => balance,
            None => 0,
        };
        let updated_balance = balance + amount;
        self.user_balances.insert(&account_id, &updated_balance);
        updated_balance
    }

    pub fn withdraw(&mut self, account_id: AccountId, amount: u128) -> u128 {
        let balance = match self.user_balances.get(&account_id) {
            Some(balance) => balance,
            None => 0u128,
        };
        let updated_balance = match balance.checked_sub(amount) {
            Some(balance) => balance,
            None => panic!(
                "Insuffcient balance to withdraw requested amount for {}!",
                account_id
            ),
        };
        self.user_balances.insert(&account_id, &updated_balance);
        if updated_balance == 0 {
            self.user_balances.remove(&account_id);
        };
        updated_balance
    }

    pub fn user_balance(&self, account_id: AccountId) -> u128 {
        return match self.user_balances.get(&account_id) {
            Some(balance) => balance,
            None => 0,
        };
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{
        get_created_receipts, get_logs, testing_env_with_promise_results, VMContextBuilder,
    };
    use near_sdk::{testing_env, Balance, MockedBlockchain, PromiseResult, VMContext};
    use std::convert::TryInto;

    fn fdai() -> AccountId {
        "fdai.testnet".to_string()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(bob().try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    // deposit with no previous balance
    #[test]
    fn deposit_no_previous_balance() {
        let context = get_context(false);
        testing_env!(context);

        let mut proposal_escrow = ProposalEscrow::new();
        let balance = proposal_escrow.deposit(bob(), 100);

        assert_eq!(balance, 100, "Balance does not equal 100");
        let balance = proposal_escrow.user_balances.get(&bob()).unwrap();
        assert_eq!(balance, 100, "Saved balance does not equal 100")
    }
    // deposit with previous balance
    #[test]
    fn deposit_with_previous_balance() {
        let context = get_context(false);
        testing_env!(context);

        let mut proposal_escrow = ProposalEscrow::new();
        proposal_escrow.user_balances.insert(&bob(), &20);
        let balance = proposal_escrow.deposit(bob(), 100);

        assert_eq!(balance, 120, "Balance does not equal 120");
        let balance = proposal_escrow.user_balances.get(&bob()).unwrap();
        assert_eq!(balance, 120, "Saved balance does not equal 120")
    }

    // Withdraw with a previous balance
    #[test]
    fn withdraw_with_previous_balance() {
        let context = get_context(false);
        testing_env!(context);

        let mut proposal_escrow = ProposalEscrow::new();
        proposal_escrow.user_balances.insert(&bob(), &20);
        let balance = proposal_escrow.withdraw(bob(), 17);

        assert_eq!(balance, 3, "Balance does not equal 3");
        let balance = proposal_escrow.user_balances.get(&bob()).unwrap();
        assert_eq!(balance, 3, "Saved balance does not equal 3")
    }

    // Withdraw without a previous balance
    #[test]
    #[should_panic(expected = r#"Insuffcient balance to withdraw requested amount for bob.near"#)]
    fn withdraw_with_no_balance() {
        let context = get_context(false);
        testing_env!(context);

        let mut proposal_escrow = ProposalEscrow::new();
        proposal_escrow.user_balances.insert(&bob(), &20);
        let balance = proposal_escrow.withdraw(bob(), 21);
    }
}
