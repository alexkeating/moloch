extern crate near_sdk;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, PanicOnDefault};

// Guild bank
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct GuildBank {
    token_id: AccountId,
}

#[near_bindgen]
impl GuildBank {
    #[init]
    pub fn new(approved_token: AccountId) -> Self {
        Self {
            token_id: approved_token,
        }
    }

    pub fn withdraw(&self, receiver: AccountId, shares: u128, total_shares: u128) -> bool {
        false
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
    fn withdraw() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = GuildBank::new(fdai());
        let withdrew = contract.withdraw(robert(), 0, 0);
        assert_eq!(withdrew, false)
    }
}
