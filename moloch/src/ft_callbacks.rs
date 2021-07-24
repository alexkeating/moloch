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

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{
        get_created_receipts, get_logs, testing_env_with_promise_results, VMContextBuilder,
    };
    use near_sdk::{testing_env, Balance, MockedBlockchain, PromiseResult, VMContext};
    use std::convert::TryInto;

    #[test]
    fn ft_on_transfer() {
        let context = VMContextBuilder::new()
            .signer_account_id("bob.near".to_string().try_into().unwrap())
            .is_view(false)
            .build();
        testing_env!(context);
        let mut contract = Moloch::new(
            "bob.near".to_string(),
            "fdau.near".to_string(),
            10.into(),
            10.into(),
            10.into(),
            10.into(),
            10.into(),
            10.into(),
            10.into(),
        );

        let promise = contract.ft_on_transfer(
            "bob.near".to_string().try_into().unwrap(),
            10.into(),
            "".to_string(),
        );
        let returned_amount = match promise {
            PromiseOrValue::Promise(_) => 0,
            PromiseOrValue::Value(T) => T.into(),
        };

        assert_eq!(returned_amount, 10, "Returned amount is incorrect");
    }
}
