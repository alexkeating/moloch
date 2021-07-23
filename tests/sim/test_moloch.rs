use near_sdk::json_types::U128;
use near_sdk_sim::{call, to_yocto, view};

use crate::utils::init_moloch;

#[test]
fn simulate_submit_proposal() {
    let (_, moloch, fdai, alice, bob, deposit_amount) = init_moloch();

    call!(
        bob,
        fdai.ft_transfer_call(
            moloch.user_account.valid_account_id(),
            deposit_amount.into(),
            Some("Deposit some tokens into moloch".to_string()),
            "".to_string()
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    call!(
        alice,
        fdai.ft_transfer_call(
            moloch.user_account.valid_account_id(),
            to_yocto("2").into(),
            Some("Deposit some tokens into moloch".to_string()),
            "".to_string()
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    call!(
        bob,
        moloch.submit_proposal(
            alice.valid_account_id().to_string(),
            to_yocto("2").into(),
            15.into(),
            "A random proposal".to_string()
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    let moloch_balance: U128 = view!(fdai.ft_balance_of(moloch.valid_account_id())).unwrap_json();
    assert_eq!(to_yocto("4"), moloch_balance.0);
}
