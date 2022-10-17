// use skw_contract_sim::{
//     call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS,
//     STORAGE_AMOUNT,
// };
// extern crate cross_contract_high_level;
// Note: the struct xxxxxxContract is created by #[skw_bindgen] from skw-sdk in combination with
// skw-sdk-sim
// use cross_contract_high_level::CrossContractContract;

// skw_contract_sim::lazy_static_include::lazy_static_include_bytes! {
//     TOKEN_WASM_BYTES => ,
// }

// fn init() -> (UserAccount, ContractAccount<CrossContractContract>) {
//     let mut genesis = skw_contract_sim::runtime::GenesisConfig::default();
//     genesis.gas_limit = u64::MAX;
//     genesis.gas_price = 0;
//     let master_account = init_simulator(Some(genesis));
//     let contract_account = deploy! {
//         contract: CrossContractContract,
//         contract_id: "contract",
//         bytes: &TOKEN_WASM_BYTES,
//         signer_account: master_account
//     };
//     (master_account, contract_account)
// }

extern crate cross_contract_high_level;
use cross_contract_high_level::CrossContractContract;


use skw_vm_interface::call::Caller;
use skw_vm_primitives::account_id::AccountId;
use skw_contract_sdk::types::AccountId as SmallAccountId;

#[test]
fn test_sim_transfer() {

    let mut caller = Caller::new_test_env(
        AccountId::testn(1),
        "./res".to_string()
    );

    let a = caller.deploy(
        include_bytes!("../res/cross_contract_high_level.wasm"),
        AccountId::testn(2),
        35_000_000_000_000_000_000u128
    );
    // println!("deploy {:?}", a);

    let contract = CrossContractContract {
        account_id: SmallAccountId::test(2)
    };

    // let aa = caller.create_user(
    //     AccountId::testn(5),
    //     1_000_000_000_000_000_000_000_000u128
    // );
    // // println!("create_user {:?}", aa);


    // let b = caller.function_call(
    //     contract.deploy_status_message(SmallAccountId::test(3), skw_contract_sdk::json_types::U128(350000000000)),
    //     0
    // );
    // // println!("function_call {:?}", b);

    // caller.set_account(AccountId::testn(5));
    // // // println!("{:?} {:?}", b, caller.state_root());


    // let c = caller.function_call(
    //     contract.complex_call(SmallAccountId::test(3), "hello".to_string()),
    //     0
    // );

    // let c = caller.call(
    //     AccountId::testn(2),
    //     "simple_call",
    //     b"{\"account_id\": [2,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5], \"message\": \"Hello\"}",
    //     300_000_000_000_000,
    //     0,
    // );

    // println!("{:?} {:?}", c, caller.state_root());
    // println!("{:?} {:?}", c.unwrap().unwrap_json_value(), caller.state_root());


    // let d = caller.view_method_call(
    //     contract.get_status_proxy(SmallAccountId::test(3), SmallAccountId::test(5))
    // );
    // println!("{:?} {:?}", d, caller.state_root());


    // let cc = caller.call(
    //     AccountId::testn(3),
    //     "set_status",
    //     b"{\"account_id\": [2,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5], \"message\": \"Yi\"}",
    //     300_000_000_000_000,
    //     0,
    // );
    // println!("{:?} {:?}", cc, caller.state_root());

    // let d = caller.view(
    //     AccountId::testn(3),
    //     "get_status",
    //     b"{\"account_id\": \"020505050505050505050505050505050505050505050505050505050505050505\"",
    // );
    // println!("{:?} {:?}", d, caller.state_root());

    // println!("{:?}", c);

    // let (master_account, contract) = init();

    // let status_id = AccountId::test(1);a
    // let status_amt = to_yocto("35");
    // call!(
    //     master_account,
    //     contract.deploy_status_message(status_id.clone(), status_amt.into()),
    //     deposit = STORAGE_AMOUNT
    // )
    // .assert_success();

    // let message = "hello world";
    // let res = call!(master_account, contract.complex_call(status_id, message.to_string()));
    // assert!(res.is_ok(), "complex_call has promise_errors: {:#?}", res.promise_results());

    // let value = res.unwrap_json_value();
    // assert_eq!(message, value.to_string().trim_matches(|c| c == '"'));
    let v: Vec<u8> = vec![7, 1,  8, 10];
    // let v: Vec<u8> = vec![11, 10];

    let s = caller.function_call( contract.merge_sort(v), 0 ).unwrap();
    println!("{:?}", s.unwrap_borsh::<Vec<u8>>());

    assert!(1 == 0);
    // call!(master_account, contract.merge_sort(v1)).assert_success();

    // let res = call!(master_account, contract.merge_sort(_v.clone()), gas = DEFAULT_GAS * 500);
    // res.assert_success();
    // let arr = res.unwrap_borsh::<Vec<u8>>();
    // let (_last, b) = arr.iter().fold((0u8, true), |(prev, b), curr| (*curr, prev <= *curr && b));
    // assert!(b, "array is not sorted.");
}
