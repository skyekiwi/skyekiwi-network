extern crate callback_results;
use callback_results::CallbackContract;


use skw_vm_interface::call::Caller;
use skw_vm_primitives::account_id::AccountId;
use skw_contract_sdk::types::AccountId as SmallAccountId;

#[test]
fn test_sim_transfer() {

    let caller = Caller::new_test_env( false, false );

    let contract = CallbackContract {
        account_id: SmallAccountId::test(2)
    };

    let res = caller.deploy(
        include_bytes!("../res/callback_results.wasm"),
        AccountId::testn(2),
        35_000_000_000_000_000_000u128
    ).unwrap();
    res.assert_success();
    
    let res = caller.function_call( contract.a(), 0, 1).unwrap();
    assert_eq!(res.unwrap_json::<u8>(), 8);

    let res = caller.function_call( contract.call_all(false, 1), 0, 1 ).unwrap();
    assert_eq!(res.unwrap_json::<(bool, bool)>(), (false, false));

    let res = caller.function_call( contract.call_all(true, 1), 0, 1 ).unwrap();
    assert_eq!(res.unwrap_json::<(bool, bool)>(), (true, false));

    let res = caller.function_call( contract.call_all(false, 0), 0, 1 ).unwrap();
    assert_eq!(res.unwrap_json::<(bool, bool)>(), (false, true));

    let res = caller.function_call( contract.call_all(true, 0), 0, 1 ).unwrap();
    assert_eq!(res.unwrap_json::<(bool, bool)>(), (true, true));
}
