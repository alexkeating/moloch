use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::json_types::U128;

use near_sdk::{env, Balance, Promise};

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
        let amount: Balance = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        if self.accounts.contains_key(&account_id) {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic(b"The attached deposit is less than the minimum storage balance");
            }

            if self
                .user_storage_accounts
                .insert(
                    account_id,
                    &StorageBalance {
                        total: amount,
                        available: amount - min_balance,
                    },
                )
                .is_some()
            {
                env::panic(b"The account is already registered");
            }
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.storage_balance_of(&account_id).unwrap()
    }

    // Send deposit, if amount is greater than deposit panic
    // if not registered panic
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        let predecessor_account_id = env::predecessor_account_id();
        let user_account = self.user_storage_accounts.get(&predecessor_account_id);
        // If not registered panic
        if user_account.is_none() {
            env::panic(
                format!("The account {} is not registered", &predecessor_account_id).as_bytes(),
            )
        };

        let storage_account = Some(user_account);

        // if amount is none transfer entire available balance
        if amount.is_none() {
            Promise:new(predecessor_account_id).transfer(storage_account.available);
            let total = storage_account.total - storage_account.available;
            let new_storage_balance = StorageBalance{total: total, available: 0}
            self.user_storage_accounts.insert(&predecessor_account_id, &new_storage_balance)
            new_storage_balance
        }

        // if amount is not None return amount
        // Existing amount path
        if amount.is_some() {
            assert!(storage_account.available >= amount, "Requested amount to withdraw is greater than the avialable amount");
            Promise:new(predecessor_account_id).transfer(amount);
            let available = storage_account.available - amount;
            let new_storage_balance = StorageBalance{total: total, available: available}
            self.user_storage_accounts.insert(&predecessor_account_id, &new_storage_balance)
            new_storage_balance
        }

        user_account

    }

    // If zero balance remove
    // IF non-zero balance force must be set to remove
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        if let Some(balance) = self.user_storage_accounts.get(&account_id) {
            if balance.available == 0 || force {
                self.accounts.remove(&account_id);
                Promise::new(account_id.clone()).transfer(balance.available + 1);
                Some((account_id, balance))
            } else {
                env::panic(b"Can't unregister the account with the positive balance without force")
            }
        } else {
            log!("The account {} is not registered", &account_id);
            None
        }
    }

    // Returns the min and max storage bounds
    // max will be null because users can continuously add proposals
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let min_required_storage_balance =
            Balance::from(self.min_account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: min_account_storage_usage,
            max: None,
        }
    }

    // Users balance for storage
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBaance> {
        self.user_accounts_storage.get(&account_id)
    }
}
