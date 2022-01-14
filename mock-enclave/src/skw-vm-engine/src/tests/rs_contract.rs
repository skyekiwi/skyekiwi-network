use skw_vm_primitives::contract_runtime::{ContractCode, Balance};
use skw_vm_primitives::fees::RuntimeFeesConfig;
use skw_vm_primitives::errors::{FunctionCallError, VMError, WasmTrap};

use skw_vm_host::mocks::mock_external::MockedExternal;
use skw_vm_host::{types::ReturnData, VMConfig, VMOutcome};

use std::mem::size_of;
use crate::runner::WasmiVM;

use crate::tests::{
    create_context, CURRENT_ACCOUNT_ID,
    PREDECESSOR_ACCOUNT_ID, SIGNER_ACCOUNT_ID, SIGNER_ACCOUNT_PK,
};

fn test_contract() -> ContractCode {
    let code = near_test_contracts::rs_contract();
    ContractCode::new(&code)
}

fn assert_run_result((outcome, err): (Option<VMOutcome>, Option<VMError>), expected_value: u64) {
    if let Some(_) = err {
        panic!("Failed execution");
    }

    if let Some(VMOutcome { return_data, .. }) = outcome {
        if let ReturnData::Value(value) = return_data {
            let mut arr = [0u8; size_of::<u64>()];
            arr.copy_from_slice(&value);
            let res = u64::from_le_bytes(arr);
            assert_eq!(res, expected_value);
        } else {
            panic!("Value was not returned");
        }
    } else {
        panic!("Failed execution");
    }
}

fn arr_u64_to_u8(value: &[u64]) -> Vec<u8> {
    let mut res = vec![];
    for el in value {
        res.extend_from_slice(&el.to_le_bytes());
    }
    res
}

#[test]
pub fn test_read_write() {
    let code = test_contract();
    let mut fake_external = MockedExternal::new();

    let context = create_context(arr_u64_to_u8(&[10u64, 20u64]));
    let config = VMConfig::test();
    let fees = RuntimeFeesConfig::test();

    let promise_results = vec![];
    let result = WasmiVM::run(
        &code,
        "write_key_value",
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    );
    assert_run_result(result, 0);

    let context = create_context(arr_u64_to_u8(&[10u64]));
    let result = WasmiVM::run(
        &code,
        "read_value",
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    );
    assert_run_result(result, 20);
}

#[test]
pub fn test_stablized_host_function() {
    let code = test_contract();
    let mut fake_external = MockedExternal::new();

    let context = create_context(vec![]);
    let config = VMConfig::test();
    let fees = RuntimeFeesConfig::test();

    let promise_results = vec![];
    let result = WasmiVM::run(
        &code,
        "do_ripemd",
        &mut fake_external,
        context.clone(),
        &config,
        &fees,
        &promise_results,
    );
    assert_eq!(result.1, None);

    let result = WasmiVM::run(
        &code,
        "do_ripemd",
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    );
    match result.1 {
        Some(VMError::FunctionCallError(FunctionCallError::LinkError { msg: _ })) => {}
        _ => panic!("should return a link error due to missing import"),
    }
}

macro_rules! def_test_ext {
    ($name:ident, $method:expr, $expected:expr, $input:expr) => {
        #[test]
        pub fn $name() {
            run_test_ext($method, $expected, $input)
        }
    };
    ($name:ident, $method:expr, $expected:expr) => {
        #[test]
        pub fn $name() {
            run_test_ext($method, $expected, &[])
        }
    };
}

fn run_test_ext(
    method: &str,
    expected: &[u8],
    input: &[u8],
) {
    let code = test_contract();
    let mut fake_external = MockedExternal::new();

    let config = VMConfig::test();
    let fees = RuntimeFeesConfig::test();
    let context = create_context(input.to_vec());

    let (outcome, err) = WasmiVM::run(
        &code,
        &method,
        &mut fake_external,
        context,
        &config,
        &fees,
        &[],
    );

    if let Some(outcome) = &outcome {
        assert_eq!(outcome.profile.action_gas(), 0);
    }

    if let Some(_) = err {
        panic!("Failed execution: {:?}", err);
    }

    if let Some(VMOutcome { return_data, .. }) = outcome {
        if let ReturnData::Value(value) = return_data {
            assert_eq!(&value, &expected);
        } else {
            panic!("Value was not returned");
        }
    } else {
        panic!("Failed execution");
    }
}

def_test_ext!(ext_account_id, "ext_account_id", CURRENT_ACCOUNT_ID.as_bytes());

def_test_ext!(ext_signer_id, "ext_signer_id", SIGNER_ACCOUNT_ID.as_bytes());
def_test_ext!(
    ext_predecessor_account_id,
    "ext_predecessor_account_id",
    PREDECESSOR_ACCOUNT_ID.as_bytes(),
    &[]
);
def_test_ext!(ext_signer_pk, "ext_signer_pk", &SIGNER_ACCOUNT_PK);
def_test_ext!(ext_random_seed, "ext_random_seed", &[0, 1, 2]);

def_test_ext!(ext_prepaid_gas, "ext_prepaid_gas", &(10_u64.pow(14)).to_le_bytes());

// TODO: change block_index to block_number
def_test_ext!(ext_block_index, "ext_block_index", &10u64.to_le_bytes());
def_test_ext!(ext_block_timestamp, "ext_block_timestamp", &42u64.to_le_bytes());
def_test_ext!(ext_storage_usage, "ext_storage_usage", &12u64.to_le_bytes());
// Note, the used_gas is not a global used_gas at the beginning of method, but instead a diff
// in used_gas for computing fib(30) in a loop
def_test_ext!(ext_used_gas, "ext_used_gas", &[111, 10, 200, 15, 0, 0, 0, 0]);
def_test_ext!(
    ext_sha256,
    "ext_sha256",
    &[
        18, 176, 115, 156, 45, 100, 241, 132, 180, 134, 77, 42, 105, 111, 199, 127, 118, 112, 92,
        255, 88, 43, 83, 147, 122, 55, 26, 36, 42, 156, 160, 158,
    ],
    b"tesdsst"
);
// current_account_balance = context.account_balance + context.attached_deposit;
def_test_ext!(ext_account_balance, "ext_account_balance", &(2u128 + 2).to_le_bytes());
def_test_ext!(ext_attached_deposit, "ext_attached_deposit", &2u128.to_le_bytes());

#[test]
pub fn test_out_of_memory() {
    let code = test_contract();
    let mut fake_external = MockedExternal::new();

    let context = create_context(Vec::new());
    let config = VMConfig::free();
    let fees = RuntimeFeesConfig::free();

    let promise_results = vec![];
    let result = WasmiVM::run(
        &code,
        "out_of_memory",
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    );
    assert_eq!(
        result.1,
        Some(VMError::FunctionCallError(
            FunctionCallError::WasmTrap(WasmTrap::Unreachable)
        ))
    );
}
