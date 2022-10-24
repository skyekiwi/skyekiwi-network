mod compile_errors;
mod rs_contract;
mod runtime_errors;
mod ts_contract;
mod wasm_validation;

// mod contract_preload;
use crate::runner::WasmiVM;

use skw_vm_primitives::contract_runtime::ContractCode;
use skw_vm_primitives::account_id::AccountId;
use skw_vm_primitives::fees::RuntimeFeesConfig;
use skw_vm_primitives::errors::VMError;

use skw_vm_host::mocks::mock_external::MockedExternal;
use skw_vm_host::{VMConfig, VMContext, VMOutcome};

fn create_context(input: Vec<u8>) -> VMContext {
    let current_account_id = AccountId::testn(1);
    let signer_account_id = AccountId::testn(2);
    let predecessor_account_id = AccountId::testn(3);

    VMContext {
        current_account_id,
        signer_account_id,
        predecessor_account_id,
        input,
        block_number: 10,
        block_timestamp: 42,
        account_balance: 2u128,
        storage_usage: 12,
        attached_deposit: 2u128,
        prepaid_gas: 10_u64.pow(14),
        random_seed: vec![0, 1, 2],
        view_config: None,
        output_data_receivers: vec![],
    }
}

fn make_simple_contract_call_with_gas_vm(
    code: &[u8],
    method_name: &str,
    prepaid_gas: u64,
) -> (Option<VMOutcome>, Option<VMError>) {
    let mut fake_external = MockedExternal::new();
    let mut context = create_context(vec![]);
    context.prepaid_gas = prepaid_gas;
    let config = VMConfig::test();
    let fees = RuntimeFeesConfig::test();

    let promise_results = vec![];

    let code = ContractCode::new(code);
    WasmiVM::run(
        &code,
        method_name,
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    )
}

fn make_simple_contract_call_vm(
    code: &[u8],
    method_name: &str,
) -> (Option<VMOutcome>, Option<VMError>) {
    make_simple_contract_call_with_gas_vm(code, method_name, 10u64.pow(14))
}

#[track_caller]
fn gas_and_error_match(
    outcome_and_error: (Option<VMOutcome>, Option<VMError>),
    expected_gas: Option<u64>,
    expected_error: Option<VMError>,
) {
    match expected_gas {
        Some(gas) => {
            let outcome = outcome_and_error.0.unwrap();
            assert_eq!(outcome.used_gas, gas, "used gas differs");
            assert_eq!(outcome.burnt_gas, gas, "burnt gas differs");
        }
        None => assert!(outcome_and_error.0.is_none()),
    }

    assert_eq!(outcome_and_error.1, expected_error);
}
