mod fixtures;
mod helpers;
mod vm_logic_builder;

use fixtures::get_context;
use helpers::*;
use skw_vm_host::types::PromiseResult;
use serde_json;
use vm_logic_builder::VMLogicBuilder;
use skw_vm_primitives::account_id::AccountId;

#[test]
fn test_promise_results() {
    let mut promise_results = vec![];
    promise_results.push(PromiseResult::Successful(b"test".to_vec()));
    promise_results.push(PromiseResult::Failed);
    promise_results.push(PromiseResult::NotReady);

    let mut logic_builder = VMLogicBuilder::default();
    logic_builder.promise_results = promise_results;
    let mut logic = logic_builder.build(get_context(vec![], false));

    assert_eq!(logic.promise_results_count(), Ok(3), "Total count of registers must be 3");
    assert_eq!(logic.promise_result(0, 0), Ok(1), "Must return code 1 on success");
    assert_eq!(logic.promise_result(1, 0), Ok(2), "Failed promise must return code 2");
    assert_eq!(logic.promise_result(2, 0), Ok(0), "Pending promise must return 3");

    let buffer = [0u8; 4];
    logic.read_register(0, buffer.as_ptr() as u64).unwrap();
    assert_eq!(&buffer, b"test", "Only promise with result should write data into register");
}

#[test]
fn test_promise_batch_action_function_call() {
    let test_account = AccountId::test().as_bytes();

    let mut logic_builder = VMLogicBuilder::default();
    let mut logic = logic_builder.build(get_context(vec![], false));
    let index = promise_create(&mut logic, &test_account, 0, 0).expect("should create a promise");

    promise_batch_action_function_call(&mut logic, 123, 0, 0)
        .expect_err("shouldn't accept not existent promise index");
    let non_receipt = logic
        .promise_and(index.to_le_bytes().as_ptr() as _, 1u64)
        .expect("should create a non-receipt promise");
    promise_batch_action_function_call(&mut logic, non_receipt, 0, 0)
        .expect_err("shouldn't accept non-receipt promise index");

    promise_batch_action_function_call(&mut logic, index, 0, 0)
        .expect("should add an action to receipt");
    let expected = serde_json::json!([
    {
        "receipt_indices":[],
        "receiver_id": AccountId::test(),
        "actions":[
            {
                "FunctionCall":{
                    "method_name":"promise_create","args":"args","gas":0,"deposit":0}},
            {
                "FunctionCall":{
                    "method_name":"promise_batch_action","args":"promise_batch_action_args","gas":0,"deposit":0}
            }
        ]
    }]);
    assert_eq!(
        &serde_json::to_string(logic_builder.ext.get_receipt_create_calls()).unwrap(),
        &expected.to_string()
    );
}

#[test]
fn test_promise_batch_action_create_account() {
    let test_account = AccountId::test().as_bytes();

    let mut logic_builder = VMLogicBuilder::default();
    let mut logic = logic_builder.build(get_context(vec![], false));
    let index = promise_create(&mut logic, &test_account, 0, 0).expect("should create a promise");

    logic
        .promise_batch_action_create_account(123)
        .expect_err("shouldn't accept not existent promise index");
    let non_receipt = logic
        .promise_and(index.to_le_bytes().as_ptr() as _, 1u64)
        .expect("should create a non-receipt promise");
    logic
        .promise_batch_action_create_account(non_receipt)
        .expect_err("shouldn't accept non-receipt promise index");
    logic
        .promise_batch_action_create_account(index)
        .expect("should add an action to create account");
    assert_eq!(logic.used_gas().unwrap(), 5084567602052);
    let expected = serde_json::json!([
        {
            "receipt_indices": [],
            "receiver_id": AccountId::test(),
            "actions": [
            {
                "FunctionCall": {
                "method_name": "promise_create",
                "args": "args",
                "gas": 0,
                "deposit": 0
                }
            },
            "CreateAccount"
            ]
        }
    ]);
    assert_eq!(
        &serde_json::to_string(logic_builder.ext.get_receipt_create_calls()).unwrap(),
        &expected.to_string()
    );
}

#[test]
fn test_promise_batch_action_deploy_contract() {
    let test_account = AccountId::test().as_bytes();

    let mut logic_builder = VMLogicBuilder::default();
    let mut logic = logic_builder.build(get_context(vec![], false));
    let index = promise_create(&mut logic, &test_account, 0, 0).expect("should create a promise");
    let code = b"sample";

    logic
        .promise_batch_action_deploy_contract(123, code.len() as u64, code.as_ptr() as _)
        .expect_err("shouldn't accept not existent promise index");
    let non_receipt = logic
        .promise_and(index.to_le_bytes().as_ptr() as _, 1u64)
        .expect("should create a non-receipt promise");
    logic
        .promise_batch_action_deploy_contract(non_receipt, code.len() as u64, code.as_ptr() as _)
        .expect_err("shouldn't accept non-receipt promise index");

    logic
        .promise_batch_action_deploy_contract(index, code.len() as u64, code.as_ptr() as _)
        .expect("should add an action to deploy contract");
    assert_eq!(logic.used_gas().unwrap(), 5262864121634);
    let expected = serde_json::json!(
      [
        {
        "receipt_indices": [],
        "receiver_id": AccountId::test(),
        "actions": [
          {
            "FunctionCall": {
              "method_name": "promise_create",
              "args": "args",
              "gas": 0,
              "deposit": 0
            }
          },
          {
            "DeployContract": {
              "code": [
                115,97,109,112,108,101
              ]
            }
          }
        ]
      }
    ]);
    assert_eq!(
        &serde_json::to_string(logic_builder.ext.get_receipt_create_calls()).unwrap(),
        &expected.to_string()
    );
}

#[test]
fn test_promise_batch_action_transfer() {
    let test_account = AccountId::test().as_bytes();

    let mut context = get_context(vec![], false);
    context.account_balance = 100;
    context.attached_deposit = 10;
    let mut logic_builder = VMLogicBuilder::default();
    let mut logic = logic_builder.build(context);
    let index = promise_create(&mut logic, &test_account, 0, 0).expect("should create a promise");

    logic
        .promise_batch_action_transfer(123, 110u128.to_le_bytes().as_ptr() as _)
        .expect_err("shouldn't accept not existent promise index");
    let non_receipt = logic
        .promise_and(index.to_le_bytes().as_ptr() as _, 1u64)
        .expect("should create a non-receipt promise");
    logic
        .promise_batch_action_transfer(non_receipt, 110u128.to_le_bytes().as_ptr() as _)
        .expect_err("shouldn't accept non-receipt promise index");

    logic
        .promise_batch_action_transfer(index, 110u128.to_le_bytes().as_ptr() as _)
        .expect("should add an action to transfer money");
    logic
        .promise_batch_action_transfer(index, 1u128.to_le_bytes().as_ptr() as _)
        .expect_err("not enough money");
    assert_eq!(logic.used_gas().unwrap(), 5356792608275);
    let expected = serde_json::json!(
    [
        {
            "receipt_indices": [],
            "receiver_id": AccountId::test(),
            "actions": [
            {
                "FunctionCall": {
                "method_name": "promise_create",
                "args": "args",
                "gas": 0,
                "deposit": 0
                }
            },
            {
                "Transfer": {
                "deposit": 110
                }
            }
            ]
        }
    ]);
    assert_eq!(
        &serde_json::to_string(logic_builder.ext.get_receipt_create_calls()).unwrap(),
        &expected.to_string()
    );
}

#[test]
fn test_promise_batch_then() {
    let mut context = get_context(vec![], false);
    context.account_balance = 100;
    let mut logic_builder = VMLogicBuilder::default();
    let mut logic = logic_builder.build(context);

    let test_account = AccountId::test().as_bytes();
    let index = promise_create(&mut logic, &test_account[..], 0, 0).expect("should create a promise");

    logic
        .promise_batch_then(123, test_account.len() as u64, test_account.as_ptr() as _)
        .expect_err("shouldn't accept non-existent promise index");
    let non_receipt = logic
        .promise_and(index.to_le_bytes().as_ptr() as _, 1u64)
        .expect("should create a non-receipt promise");
    logic
        .promise_batch_then(non_receipt, test_account.len() as u64, test_account.as_ptr() as _)
        .expect("should accept non-receipt promise index");

    logic
        .promise_batch_then(index, test_account.len() as u64, test_account.as_ptr() as _)
        .expect("promise batch should run ok");
    assert_eq!(logic.used_gas().unwrap(), 24153356255723);
    let expected = serde_json::json!([
        {
            "receipt_indices": [],
            "receiver_id": AccountId::test(),
            "actions": [
                {
                    "FunctionCall": {
                        "method_name": "promise_create",
                        "args": "args",
                        "gas": 0,
                        "deposit": 0
                    }
                }
            ]
        },
        {
            "receipt_indices": [
                0
            ],
            "receiver_id": AccountId::test(),
            "actions": []
        },
        {
            "receipt_indices": [
                0
            ],
            "receiver_id": AccountId::test(),
            "actions": []
        }
    ]);
    assert_eq!(
        &serde_json::to_string(logic_builder.ext.get_receipt_create_calls()).unwrap(),
        &expected.to_string()
    );
}
