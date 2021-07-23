extern crate near_sdk;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ProposalEscrow {
    token_id: AccountId,
    user_balances: UnorderedMap<AccountId, u128>,
}

// This can only be called internally
// on the token Receiver
impl ProposalEscrow {
    pub fn new(approved_token: AccountId) -> Self {
        Self {
            token_id: approved_token,
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
        println!("Balance {}", amount);
        println!("Balance {}", balance);
        let updated_balance = match balance.checked_sub(amount) {
            Some(balance) => balance,
            None => panic!(
                "Insuffcient balance to withdraw requested amount for {}!",
                account_id
            ),
        };
        self.user_balances.insert(&account_id, &updated_balance);
        updated_balance
    }
}
