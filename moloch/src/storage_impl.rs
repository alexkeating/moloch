use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::json_types::U128;

// So now a user will need to register to
// moloch before they can send fungible tokens
//
// We will change unordered maps to looup maps where
// necessary because they cost less gas to store
//
// User accounts will live on Moloch and we can split to internal logic
impl StorageManagent for Moloch {
    // Add assigned balance to a lookup map
    // If already exists add to the balance
    // If less than minimum amount panic
    //
    // Does registration only need to be implemented
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
    }

    // Send deposit, if amount is greater than deposit panic
    // if not registered panic
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {}

    // If zero balance remove
    // IF non-zero balance force must be set to remove
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {}

    // Returns the min and max storage bounds
    // max will be null because users can continuously add proposals
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {}

    // Users balance for storage
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBaance> {}
}
