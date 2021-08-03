use moloch::MolochContract;
use near_sdk::serde_json::json;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use test_fungible_token::ContractContract as FdaiContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    MOLOCH_WASM_BYTES => "res/moloch.wasm",
    FDAI_WASM_BYTES => "res/test_fungible_token.wasm"
}

const MOLOCH_ID: &str = "moloch";
const FDAI_ID: &str = "fdai";

// Register the given `user` with FT contract
pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        FDAI_ID.parse().unwrap(),
        "storage_deposit",
        &json!({
            "account_id": user.account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 125, // attached deposit
    )
    .assert_success();
}

pub fn register_user_moloch(
    user: &near_sdk_sim::UserAccount,
    moloch: &near_sdk_sim::ContractAccount<MolochContract>,
) {
    call!(
        user,
        moloch.storage_deposit(None, Some(false)),
        to_yocto("12"),
        near_sdk_sim::DEFAULT_GAS
    );
}

pub fn init_moloch() -> (
    UserAccount,
    ContractAccount<MolochContract>,
    ContractAccount<FdaiContract>,
    UserAccount,
    UserAccount,
    u128,
) {
    let root = init_simulator(None);

    let fdai = deploy!(
       contract: FdaiContract,
       contract_id: FDAI_ID,
       bytes: &FDAI_WASM_BYTES,
       signer_account: root,
       init_method: new_default_meta(root.valid_account_id(), to_yocto("900").into())
    );

    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    let bob = root.create_user("bob".to_string(), to_yocto("100"));
    register_user(&alice);
    register_user(&bob);
    register_user(&root);
    let deposit_amount = to_yocto("2");

    println!("Account Id");
    println!("{:?}", fdai.user_account.account_id.to_string());
    let moloch = deploy!(
        contract: MolochContract,
        contract_id: MOLOCH_ID,
        bytes: &MOLOCH_WASM_BYTES,
        signer_account: root,
        init_method: new(
            bob.valid_account_id().to_string(),
            fdai.user_account.account_id.to_string(),
            // nanoseconds
             10u64.pow(9).into(),
             3u64.into(),
             1u64.into(),
             2u64.into(),
             deposit_amount.into(),
             2u128.into(),
             1u128.into()
             )
    );

    register_user(&moloch.user_account);

    call!(
        root,
        fdai.ft_transfer(alice.valid_account_id(), to_yocto("100").into(), None),
        deposit = 1
    )
    .assert_success();

    call!(
        root,
        fdai.ft_transfer(bob.valid_account_id(), to_yocto("100").into(), None),
        deposit = 1
    )
    .assert_success();

    (root, moloch, fdai, alice, bob, deposit_amount)
}
