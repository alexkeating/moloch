use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::json_types::{ValidAccountId, U128};
use std::convert::TryFrom;

use near_sdk::{assert_one_yocto, env, Balance, Promise};

// So now a user will need to register to
// moloch before they can send fungible tokens
//
// We will change unordered maps to looup maps where
// necessary because they cost less gas to store
//
// User accounts will live on Moloch and we can split to internal logic
impl StorageManagement for Moloch {
    // Add assigned balance to a lookup map
    // If already exists add to the balance
    // If less than minimum amount panic
    //
    // Does registration only need to be implemented
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        let account_id = match account_id {
            Some(account_id) => account_id.to_string(),
            None => env::predecessor_account_id(),
        };
        let valid_account_id = ValidAccountId::try_from(account_id.to_string()).unwrap();

        let mut user_storage = UserStorageBalance {
            total: 0,
            available: 0,
        };
        let min_balance = self.storage_balance_bounds().min.0;
        let user_storage_opt = self.user_storage_accounts.get(&account_id);
        if user_storage_opt.is_some() {
            user_storage = self.user_storage_accounts.get(&account_id).unwrap();
        } else {
            if amount < min_balance {
                env::panic(b"The attached deposit is less than the minimum storage balance bounds");
            }
        };
        if registration_only.is_none() || registration_only.unwrap() == false {
            let mut available = user_storage.available + amount;
            if user_storage_opt.is_none() {
                available = amount - u128::from(min_balance);
            }

            self.user_storage_accounts.insert(
                &account_id,
                &UserStorageBalance {
                    total: user_storage.total + amount,
                    available: available,
                },
            );
            return self.storage_balance_of(valid_account_id).unwrap();
        }

        // If account already registered refund the full amount
        if user_storage_opt.is_some() {
            env::log("The account is already registered, refunding the deposit".as_bytes());
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
            return self.storage_balance_of(valid_account_id).unwrap();
        }

        // If registration true refund above minimum
        self.user_storage_accounts.insert(
            &account_id,
            &UserStorageBalance {
                total: min_balance,
                available: 0,
            },
        );

        let refund = amount - min_balance;
        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
        self.storage_balance_of(valid_account_id).unwrap()
    }

    // Send deposit, if amount is greater than deposit panic
    // if not registered panic
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        let user_account = self.user_storage_accounts.get(&predecessor_account_id);
        // If not registered panic
        if user_account.is_none() {
            env::panic(
                format!("The account {} is not registered", &predecessor_account_id).as_bytes(),
            )
        };

        let storage_account = user_account.unwrap();

        // if amount is none transfer entire available balance
        if amount.is_none() {
            Promise::new(predecessor_account_id.to_string()).transfer(storage_account.available);
            let total = storage_account.total - storage_account.available;
            let new_storage_balance = UserStorageBalance {
                total: total,
                available: 0,
            };
            self.user_storage_accounts
                .insert(&predecessor_account_id, &new_storage_balance);
            return StorageBalance {
                total: new_storage_balance.total.into(),
                available: new_storage_balance.available.into(),
            };
        }

        // if amount is not None return amount
        // Existing amount path
        let amount = amount.unwrap();
        assert!(
            storage_account.available >= amount.into(),
            "Requested amount to withdraw is greater than the available amount to withdraw"
        );
        Promise::new(predecessor_account_id.to_string()).transfer(amount.into());
        let available = u128::from(storage_account.available) - u128::from(amount);
        let total = u128::from(storage_account.total) - u128::from(amount);
        let new_storage_balance = StorageBalance {
            total: total.into(),
            available: available.into(),
        };
        self.user_storage_accounts.insert(
            &predecessor_account_id,
            &UserStorageBalance {
                total: new_storage_balance.total.into(),
                available: new_storage_balance.available.into(),
            },
        );
        new_storage_balance
    }

    // If zero balance remove
    // IF non-zero balance force must be set to remove
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        if let Some(balance) = self.user_storage_accounts.get(&account_id) {
            if balance.available == 0 || force {
                self.user_storage_accounts.remove(&account_id);
                Promise::new(account_id.clone()).transfer(balance.available + 1);
                true
            } else {
                env::panic(b"Can't unregister the account with a positive balance without a force")
            }
        } else {
            env::log(format!("The account {} is not registered", &account_id).as_bytes());
            false
        }
    }

    // Returns the min and max storage bounds
    // max will be null because users can continuously add proposals
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let min_required_storage_balance =
            Balance::from(self.min_account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: u128::from(min_required_storage_balance).into(),
            max: None,
        }
    }

    // Users balance for storage
    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        match self.user_storage_accounts.get(&account_id.to_string()) {
            Some(user_storage_balance) => Some(StorageBalance {
                total: user_storage_balance.total.into(),
                available: user_storage_balance.available.into(),
            }),
            None => None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::{
        alice, bob, get_context, get_context_builder, robert, storage_deposit, MockMember,
        MockMoloch, MockProposal,
    };
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};
    use std::convert::TryInto;

    // If account id omitted then add deposit to predecessor_account (sim)
    // If account id is included then add to the account id (sim)
    // If registration only true and amount is over minimum then refund the minimum (sim)
    // If registration only and already registered then refund the full amount (sim)
    // If registration and exact amount then refund nothing
    //
    // Default account id
    // passed in account id

    // storage_deposit
    #[test]
    fn storage_deposit_registration_default() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().build();
        testing_env!(context_builder
            .attached_deposit(60000000000000000000)
            .build());
        contract.storage_deposit((None), Some(false));
        let storage_balance = contract.user_storage_accounts.get(&bob()).unwrap();
        assert_eq!(
            storage_balance.total, 60000000000000000000,
            "Total deposit is not correct"
        );
        assert_eq!(
            storage_balance.available, 10000000000000000000,
            "Availble deposit is not correct"
        );
    }

    //  registration none
    #[test]
    fn storage_deposit_registration_none() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().build();
        testing_env!(context_builder
            .attached_deposit(60000000000000000000)
            .build());
        let storage_balance = contract.storage_deposit(Some(bob().try_into().unwrap()), None);
        assert_eq!(
            u128::from(storage_balance.total),
            60000000000000000000,
            "Total deposit is not correct"
        );
        assert_eq!(
            u128::from(storage_balance.available),
            10000000000000000000,
            "Availble deposit is not correct"
        );
    }

    //  registration false exists
    #[test]
    fn storage_deposit_registration_false_exists() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new()
            .register_user(bob(), 50000000000000000000, 0)
            .build();
        testing_env!(context_builder.attached_deposit(5).build());
        let storage_balance =
            contract.storage_deposit(Some(bob().try_into().unwrap()), Some(false));
        assert_eq!(
            u128::from(storage_balance.total),
            50000000000000000005,
            "Total deposit is not correct"
        );
        assert_eq!(
            u128::from(storage_balance.available),
            5,
            "Availble deposit is not correct"
        );
    }

    //  registration exists
    #[test]
    fn storage_deposit_registration_exists() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 5, 5).build();
        testing_env!(context_builder
            .attached_deposit(50000000000000000000)
            .build());
        let storage_balance = contract.storage_deposit(Some(bob().try_into().unwrap()), Some(true));
        assert_eq!(
            u128::from(storage_balance.total),
            5,
            "Total deposit is not correct"
        );
        assert_eq!(
            u128::from(storage_balance.available),
            5,
            "Availble deposit is not correct"
        );
    }

    //  registration less than min
    #[test]
    #[should_panic(
        expected = r#"The attached deposit is less than the minimum storage balance bounds"#
    )]
    fn storage_deposit_registration_less_than_min() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().build();
        testing_env!(context_builder.attached_deposit(5).build());
        let storage_balance = contract.storage_deposit(Some(bob().try_into().unwrap()), Some(true));
    }

    //  registration more than the amount
    #[test]
    fn storage_deposit_registration_more_than_minimum() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().build();
        testing_env!(context_builder
            .attached_deposit(60000000000000000000)
            .build());
        contract.storage_deposit(None, Some(true));
        let storage_balance = contract.user_storage_accounts.get(&bob()).unwrap();
        assert_eq!(
            storage_balance.total, 50000000000000000000,
            "Total deposit is not correct"
        );
        assert_eq!(
            storage_balance.available, 0,
            "Availble deposit is not correct"
        );
    }

    // storage_withdraw
    //
    // If no amount than the full amount is refunded
    #[test]
    fn storage_withdraw_full_refund() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 10, 5).build();
        testing_env!(context_builder.attached_deposit(1).build());
        let new_balance = contract.storage_withdraw(None);
        let storage_balance = contract.user_storage_accounts.get(&bob()).unwrap();
        assert_eq!(
            u128::from(new_balance.total),
            5,
            "Total storage was reduced in excess"
        );
        assert_eq!(
            u128::from(new_balance.available),
            0,
            "Available storage is incorrect"
        );
        assert_eq!(
            storage_balance.total, 5,
            "Total storage interally is incorrect"
        );
        assert_eq!(
            storage_balance.available, 0,
            "Available storage internally is incorrect"
        );
    }

    // If amount specified is okay then withdraw that amount
    #[test]
    fn storage_withdraw_reasonable_amount() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 10, 5).build();
        testing_env!(context_builder.attached_deposit(1).build());
        let new_balance = contract.storage_withdraw(Some(3.into()));
        let storage_balance = contract.user_storage_accounts.get(&bob()).unwrap();
        assert_eq!(
            u128::from(new_balance.total),
            7,
            "Total storage was reduced in excess"
        );
        assert_eq!(
            u128::from(new_balance.available),
            2,
            "Available storage is incorrect"
        );
        assert_eq!(
            storage_balance.total, 7,
            "Total storage interally is incorrect"
        );
        assert_eq!(
            storage_balance.available, 2,
            "Available storage internally is incorrect"
        );
    }

    // If amount specified is greater than available amount panic
    #[test]
    #[should_panic(
        expected = r#"Requested amount to withdraw is greater than the available amount to withdraw"#
    )]
    fn storage_withdraw_greater_than_available() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 10, 5).build();
        testing_env!(context_builder.attached_deposit(1).build());
        contract.storage_withdraw(Some(10.into()));
    }

    // If not registered panic
    #[test]
    #[should_panic(expected = r#"The account bob.near is not registered"#)]
    fn storage_withdraw_not_registered() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().build();
        testing_env!(context_builder.attached_deposit(1).build());
        contract.storage_withdraw(Some(10.into()));
    }

    // Require 1 yoct_neaer
    #[test]
    #[should_panic(expected = r#"Requires attached deposit of exactly 1 yoctoNEAR"#)]
    fn storage_withdraw_no_yocto() {
        let context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 5, 1).build();
        testing_env!(context_builder.build());
        contract.storage_withdraw(Some(10.into()));
    }

    // If account is not registered then the function must return false
    #[test]
    fn storage_unregisted_not_registered() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().build();
        testing_env!(context_builder.attached_deposit(1).build());
        let unregistered = contract.storage_unregister(Some(false));
        assert_eq!(unregistered, false, "User is registered");
    }

    #[test]
    fn storage_unregisted_none() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 5, 0).build();
        testing_env!(context_builder.attached_deposit(1).build());
        let unregistered = contract.storage_unregister(None);
        assert_eq!(unregistered, true, "Did not register without force");
    }

    // If force is true and account exists than non-zero accounts should be ignored and
    // the account is removed
    #[test]
    fn storage_unregisted_with_force() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 5, 5).build();
        testing_env!(context_builder.attached_deposit(1).build());
        let unregistered = contract.storage_unregister(Some(true));
        assert_eq!(unregistered, true, "User was not unregistered");
        let user = contract.user_storage_accounts.get(&bob());
        assert_eq!(user.is_none(), true, "User is still registered");
    }

    // If caller has a non zero balance without a force than panic
    #[test]
    #[should_panic(
        expected = r#"Can't unregister the account with a positive balance without a force"#
    )]
    fn storage_unregisted_no_force() {
        let mut context_builder = get_context_builder(false);
        testing_env!(context_builder.build());
        let mut contract = MockMoloch::new().register_user(bob(), 5, 5).build();
        testing_env!(context_builder.attached_deposit(1).build());
        contract.storage_unregister(Some(false));
    }

    // storage_unregister

    // Call and get back expected balance
    #[test]
    fn storage_balance_bounds() {
        let context = get_context(false);
        testing_env!(context);
        let contract = MockMoloch::new().min_account_storage_usage(20).build();
        let storage_balance_bounds = contract.storage_balance_bounds();
        assert_eq!(storage_balance_bounds.max, None, "Max storage is not None");
        assert_eq!(
            storage_balance_bounds.min,
            U128::from(200000000000000000000),
            "Min storage is not correct"
        );
    }

    // User acount exists
    #[test]
    fn storage_balance_of_exists() {
        let context = get_context(false);
        testing_env!(context);
        let contract = MockMoloch::new().register_user(bob(), 5, 5).build();
        let balanace = contract
            .storage_balance_of(bob().try_into().unwrap())
            .unwrap();
        assert_eq!(
            u128::from(balanace.total),
            5,
            "User storage total balance is incorrect"
        );
        assert_eq!(
            u128::from(balanace.available),
            5,
            "User storage available balance is incorrect"
        );
    }

    // User account does not exist
    #[test]
    fn storage_balance_of_does_not_exists() {
        let context = get_context(false);
        testing_env!(context);
        let contract = MockMoloch::new().build();
        let balance = contract.storage_balance_of(bob().try_into().unwrap());
        assert_eq!(balance.is_none(), true, "User storage is not None");
    }
}
