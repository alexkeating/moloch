use moloch::MolochContract;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use test_fungible_token::ContractContract as FdaiContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    MOLOCH_WASM_BYTES => "res/moloch.wasm",
    FDAI_WASM_BYTES => "res/test_fungible_token.wasm"
}

const MOLOCH_ID: &str = "moloch";
const FDAI_ID: &str = "fdai";

pub fn init_moloch() -> (
    UserAccount,
    ContractAccount<MolochContract>,
    ContractAccount<FdaiContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    let bob = root.create_user("bob".to_string(), to_yocto("100"));

    let fdai = deploy!(
       contract: FdaiContract,
       contract_id: FDAI_ID,
       bytes: &FDAI_WASM_BYTES,
       signer_account: root,
       init_method: new_default_meta(root.valid_account_id(), 1000.into())
    );

    let moloch = deploy!(
        contract: MolochContract,
        contract_id: MOLOCH_ID,
        bytes: &MOLOCH_WASM_BYTES,
        signer_account: root,
        init_method: new(
            root.valid_account_id().to_string(),
            fdai.user_account.account_id.to_string(),
            // nanoseconds
             10u128.pow(9).into(),
             3u128.into(),
             1u128.into(),
             2u128.into(),
             10u128.into(),
             2u128.into(),
             1u128.into()
             )
    );

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

    (root, moloch, fdai, alice, bob)
}
