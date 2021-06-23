extern crate near_sdk;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

// Guild bank
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Debug)]
pub struct GuildBank {
    token_id: AccountId,
}

impl GuildBank {
    pub fn new(approved_token: AccountId) -> Self {
        Self {
            token_id: approved_token,
        }
    }

    pub fn withdraw(&self, _receiver: AccountId, _shares: u128, _total_shares: u128) -> bool {
        false
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
        let withdrew = contract.withdraw(robert(), 0, 0);
        assert_eq!(withdrew, false)
    }
}
