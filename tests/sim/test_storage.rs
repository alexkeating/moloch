use crate::utils::init_moloch;
use near_contract_standards::storage_management::StorageBalanceBounds;
use near_sdk::env;

use near_sdk_sim::{call, to_yocto, view};
use std::convert::TryInto;
// Verify the transfers are correct
//
// storage deposit
//
// already registered full refund
#[test]
fn simulate_storage_deposit_exact() {
    let (_, moloch, fdai, alice, bob, deposit_amount) = init_moloch();
    let start_amount = bob.account().unwrap().amount;
    let min_deposit = to_yocto("7");
    println!("Here {:?} 23", start_amount);
    let res = call!(
        bob,
        moloch.storage_deposit(
            Some(bob.account_id.to_string().try_into().unwrap()),
            Some(false)
        ),
        min_deposit,
        near_sdk_sim::DEFAULT_GAS
    );
    let end_amount = bob.account().unwrap().amount;
    assert!(
        start_amount - end_amount >= to_yocto("7"),
        "Did not take all of the registration"
    );
}

#[test]
fn simulate_storage_deposit_transfer_back() {
    let (_, moloch, fdai, alice, bob, deposit_amount) = init_moloch();
    let start_amount = &bob.account().unwrap().amount;
    let deposit = 828593677552200000000;
    println!("Here {:?} 23", start_amount);
    let res = call!(
        bob,
        moloch.storage_deposit(
            Some(bob.account_id.to_string().try_into().unwrap()),
            Some(true)
        ),
        deposit,
        near_sdk_sim::DEFAULT_GAS
    );
    let end_amount = bob.account().unwrap().amount;
    assert!(
        (start_amount - end_amount) < deposit,
        "Not receieve correct excess amount"
    );
    assert!(
        (start_amount - end_amount) > 1000000,
        "Not receieve correct excess amount"
    );
}

#[test]
fn simulate_storage_deposit_already_registered() {
    let (_, moloch, _, _, bob, _) = init_moloch();
    let deposit = to_yocto("9");
    call!(
        bob,
        moloch.storage_deposit(
            Some(bob.account_id.to_string().try_into().unwrap()),
            Some(false)
        ),
        deposit,
        near_sdk_sim::DEFAULT_GAS
    );
    let start_amount = &bob.account().unwrap().amount;
    call!(
        bob,
        moloch.storage_deposit(
            Some(bob.account_id.to_string().try_into().unwrap()),
            Some(true)
        ),
        deposit,
        near_sdk_sim::DEFAULT_GAS
    );
    let end_amount = bob.account().unwrap().amount;
    println!("Difference {}", (start_amount - end_amount));
    println!("Start {}", (start_amount));
    println!("End {}", (end_amount));
    assert!(
        (start_amount - end_amount) < 800000000000000000000,
        "Not receieve correct excess amount"
    );
}
#[test]
#[should_panic(
    expected = r#"The attached deposit is less than the minimum storage balance bounds"#
)]
fn simulate_storage_deposit_below_min_amount() {
    let (_, moloch, _, alice, _, _) = init_moloch();
    let deposit = 1000;
    call!(
        alice,
        moloch.storage_deposit(
            Some(alice.account_id.to_string().try_into().unwrap()),
            Some(true)
        ),
        deposit,
        near_sdk_sim::DEFAULT_GAS
    );
}

// registration only above min refund
