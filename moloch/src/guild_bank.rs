extern crate near_sdk;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

// Guild bank
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Debug)]
pub struct GuildBank {
    token_id: AccountId,
    balance: u128,
}

impl GuildBank {
    pub fn new(approved_token: AccountId) -> Self {
        Self {
            token_id: approved_token,
            balance: 0,
        }
    }

    pub fn withdraw(&self, receiver: AccountId, shares: u128, total_shares: u128) {
        let amount = match self
            .balance
            .saturating_mul(shares)
            .checked_div(total_shares)
        {
            Some(amount) => amount,
            None => panic!("Total shares is 0 a withdrawl cannot be calculated"),
        };
        env::log(format!("Withdrawl: receiver: {}, amount: {}", receiver, amount).as_bytes());
        let prepaid_gas = env::prepaid_gas();
        ext_fungible_token::ft_transfer(
            receiver,
            U128::from(amount),
            Some("Withdrawl from guild bank".to_string()),
            &self.token_id,
            0,
            prepaid_gas / 2,
        );
    }

    pub fn deposit(&mut self, amount: u128) -> u128 {
        self.balance += amount;
        return self.balance;
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
    fn withdraw() {
        let context = get_context(false);
        testing_env!(context);
        let contract = GuildBank::new(fdai());
        contract.withdraw(robert(), 0, 1);
    }
}