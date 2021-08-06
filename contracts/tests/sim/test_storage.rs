use crate::utils::init_moloch;

use near_sdk_sim::{call, to_yocto};
use std::convert::TryInto;
// Verify the transfers are correct
//
// storage deposit
//
// already registered full refund
#[test]
fn simulate_storage_deposit_exact() {
    let (_, moloch, _fdai, _alice, bob, _deposit_amount) = init_moloch();
    let start_amount = bob.account().unwrap().amount;
    let min_deposit = to_yocto("7");
    println!("Here {:?} 23", start_amount);
    call!(
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
    let (_, moloch, _fdai, _alice, bob, _deposit_amount) = init_moloch();
    let start_amount = &bob.account().unwrap().amount;
    let deposit = 828593677552200000000;
    println!("Here {:?} 23", start_amount);
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
    assert!(
        (start_amount - end_amount) < 800000000000000000000,
        "Not receieve correct excess amount"
    );
}
#[test]
fn simulate_storage_deposit_below_min_amount() {
    let (_, moloch, _, alice, _, _) = init_moloch();
    let deposit = 0;
    let res = call!(
        alice,
        moloch.storage_deposit(
            Some(alice.account_id.to_string().try_into().unwrap()),
            Some(true)
        ),
        deposit,
        near_sdk_sim::DEFAULT_GAS
    );
    assert!(
        format!("{:?}", res.status())
            .contains("The attached deposit is less than the minimum storage balance bounds"),
        "Corrrect error was not raised"
    )
}

// Requested amount is greater than deposited
#[test]
fn simulate_storage_withdraw_and_unregister() {
    let (_, moloch, _, alice, _, _) = init_moloch();
    let deposit = to_yocto("10");
    let start_amount = &alice.account().unwrap().amount;
    call!(
        alice,
        moloch.storage_deposit(
            Some(alice.account_id.to_string().try_into().unwrap()),
            Some(false)
        ),
        deposit,
        near_sdk_sim::DEFAULT_GAS
    );
    let res = call!(
        alice,
        moloch.storage_withdraw(Some(to_yocto("9").into())),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    let end_amount = &alice.account().unwrap().amount;
    println!("Resp {:?}", res);
    println!("Diff {}", (start_amount - end_amount));
    println!("yocto {}", to_yocto("1.1"));
    assert!((start_amount - end_amount) < to_yocto("1.1"));
    let res = call!(
        alice,
        moloch.storage_unregister(Some(true)),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    let end_amount = &alice.account().unwrap().amount;
    println!("Resp {:?}", res);
    println!("Diff {}", (start_amount - end_amount));
    println!("yocto {}", 6000000000000000000000u128); // slightly above minimum storage
    assert!((start_amount - end_amount) < 6000000000000000000000);
}

// amount sent back is correct
