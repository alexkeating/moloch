use crate::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::near_bindgen;
use near_sdk::{env, PromiseOrValue};

#[near_bindgen]
impl FungibleTokenReceiver for Moloch {
    /// Deposit a transfer into the guild bank escrow
    /// As long as the sent token matches the approved token
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        if token_id == self.token_id {
            self.escrow.deposit(sender_id.into(), u128::from(amount));
            return PromiseOrValue::Value(U128(0));
        };
        PromiseOrValue::Value(amount)
    }
}
