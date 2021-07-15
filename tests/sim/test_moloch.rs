use near_sdk::json_types::U128;
use near_sdk_sim::{call, view};

use crate::utils::init_moloch;

#[test]
fn simulate_submit_proposal() {
    let (root, moloch, fdai, alice, _) = init_moloch();

    call!(
        root,
        moloch.submit_proposal(
            alice.valid_account_id().to_string(),
            15,
            15,
            "A random proposal".to_string()
        ),
        deposit = 1
    )
    .assert_success();

    let moloch_balance: U128 = view!(fdai.ft_balance_of(moloch.valid_account_id())).unwrap_json();

    assert_eq!(10, moloch_balance.0);

    // Call submit proposal and then make sure the appropriate amount is
    // deposited
    //
    // call!(
    //     root,
    //     moloch.submit_proposal(alice.valid_account_id(), transfer_amount.into(), None),
    //     deposit = 1
    // )
    // .assert_success();

    // let root_balance: U128 = view!(ft.ft_balance_of(root.valid_account_id())).unwrap_json();
    // let alice_balance: U128 = view!(ft.ft_balance_of(alice.valid_account_id())).unwrap_json();
    // assert_eq!(initial_balance - transfer_amount, root_balance.0);
    // assert_eq!(transfer_amount, alice_balance.0);
}
