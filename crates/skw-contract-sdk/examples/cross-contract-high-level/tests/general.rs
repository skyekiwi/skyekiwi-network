extern crate cross_contract_high_level;
use cross_contract_high_level::CrossContractContract;


use skw_vm_interface::call::Caller;
use skw_vm_primitives::account_id::AccountId;
use skw_contract_sdk::types::AccountId as SmallAccountId;

#[test]
fn test_sim_transfer() {

    let mut caller = Caller::new_test_env( false, false );

    let contract = CrossContractContract {
        account_id: SmallAccountId::test(2)
    };

    let res = caller.deploy(
        include_bytes!("../res/cross_contract_high_level.wasm"),
        AccountId::testn(2),
        35_000_000_000_000_000_000u128
    ).unwrap();
    res.assert_success();

    let res = caller.create_user(AccountId::testn(5), 1_000_000_000u128).unwrap();
    res.assert_success();
    
    let res = caller.function_call(
        contract.deploy_status_message(SmallAccountId::test(3), skw_contract_sdk::json_types::U128(350000000000)),
        0, 1
    ).unwrap();
    res.assert_success();

    caller.set_account(AccountId::testn(5));

    let res = caller.function_call(
        contract.complex_call(SmallAccountId::test(3), "hello".to_string()),
        0, 1
    ).unwrap();
    // println!("{:?}", res);

    res.assert_success();

    let v: Vec<u8> = vec![7, 1,  8, 10, 11, 23];
    let res = caller.function_call( contract.merge_sort(v), 0, 500 ).unwrap();
    // println!("{:?}", res.unwrap_borsh::<Vec<u8>>());

    res.assert_success();
}
