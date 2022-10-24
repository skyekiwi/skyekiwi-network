use crate::runtime_group_tools::RuntimeGroup;
use skw_vm_host::types::AccountId;
use skw_vm_primitives::contract_runtime::{CryptoHash};
use skw_vm_primitives::receipt::{ActionReceipt, ReceiptEnum};
use skw_vm_primitives::transaction::{SignedTransaction};
use skw_vm_primitives::transaction::*;

pub mod runtime_group_tools;

/// Initial balance used in tests.
pub const TESTING_INIT_BALANCE: u128 = 1_000_000_000 * NEAR_BASE;

/// One NEAR, divisible by 10^24.
pub const NEAR_BASE: u128 = 1_000_000_000_000_000_000_000_000;

const GAS_1: u64 = 900_000_000_000_000;
const GAS_2: u64 = GAS_1 / 3;
const GAS_3: u64 = GAS_2 / 3;

#[test]
fn test_simple_func_call() {
    let group = RuntimeGroup::new(2, 2, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();

    let account_ids = group.account_ids.clone();

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "sum_n".to_string(),
            args: 10u64.to_le_bytes().to_vec(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0] => r0 @ account_ids[1],
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{..}), {}
                     => [ref1] );
    assert_refund!(group, ref1 @ account_ids[0]);
}

// single promise, no callback (A->B)
#[test]
fn test_single_promise_no_callback() {
    let group = RuntimeGroup::new(3, 3, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"create": {
        "account_id": account_ids[2],
        "method_name": "call_promise",
        "arguments": [],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 0 }
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, ref0] );
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref1]);
    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
}

// single promise with callback (A->B=>C)
#[test]
fn test_single_promise_with_callback() {
    let group = RuntimeGroup::new(4, 4, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"create": {
        "account_id": account_ids[2].clone(),
        "method_name": "call_promise",
        "arguments": [],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 0 },
        {"then": {
        "promise_index": 0,
        "account_id": account_ids[3].clone(),
        "method_name": "call_promise",
        "arguments": [],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 1}
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, r2, ref0] );
    let data_id;
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, output_data_receivers, ..}), {
                        assert_eq!(output_data_receivers.len(), 1);
                        data_id = output_data_receivers[0].data_id;
                     },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref1]);
    assert_receipts!(group, account_ids[1].clone() => r2 @ account_ids[3].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, input_data_ids, ..}), {
                        assert_eq!(input_data_ids.len(), 1);
                        assert_eq!(data_id, input_data_ids[0].clone());
                     },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref2]);

    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
    assert_refund!(group, ref2 @ account_ids[0].clone());
}

// two promises, no callbacks (A->B->C)
#[test]
fn test_two_promises_no_callbacks() {
    let group = RuntimeGroup::new(4, 4, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"create": {
        "account_id": account_ids[2].clone(),
        "method_name": "call_promise",
        "arguments": [
            {"create": {
            "account_id": account_ids[3].clone(),
            "method_name": "call_promise",
            "arguments": [],
            "amount": "0",
            "gas": GAS_3,
            }, "id": 0}
        ],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 0 },

    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, ref0] );
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), { },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [r2, ref1]);
    assert_receipts!(group, account_ids[2].clone() => r2 @ account_ids[3].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_3);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref2]);

    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
    assert_refund!(group, ref2 @ account_ids[0].clone());
}

// two promises, with two callbacks (A->B->C=>D=>E) where call to E is initialized by completion of D.
#[test]
fn test_two_promises_with_two_callbacks() {
    let group = RuntimeGroup::new(6, 6, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"create": {
        "account_id": account_ids[2].clone(),
        "method_name": "call_promise",
        "arguments": [
            {"create": {
            "account_id": account_ids[3].clone(),
            "method_name": "call_promise",
            "arguments": [],
            "amount": "0",
            "gas": GAS_3,
            }, "id": 0},

            {"then": {
            "promise_index": 0,
            "account_id": account_ids[4].clone(),
            "method_name": "call_promise",
            "arguments": [],
            "amount": "0",
            "gas": GAS_3,
            }, "id": 1}
        ],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 0 },

        {"then": {
        "promise_index": 0,
        "account_id": account_ids[5].clone(),
        "method_name": "call_promise",
        "arguments": [],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 1}
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, cb1, ref0] );
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), { },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [r2, cb2, ref1]);
    assert_receipts!(group, account_ids[2].clone() => r2 @ account_ids[3].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_3);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref2]);
    assert_receipts!(group, account_ids[2].clone() => cb2 @ account_ids[4].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), { },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_3);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref3]);
    assert_receipts!(group, account_ids[1].clone() => cb1 @ account_ids[5].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), { },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref4]);

    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
    assert_refund!(group, ref2 @ account_ids[0].clone());
    assert_refund!(group, ref3 @ account_ids[0].clone());
    assert_refund!(group, ref4 @ account_ids[0].clone());
}

// Batch actions tests

// single promise, no callback (A->B) with `promise_batch`
#[test]
fn test_single_promise_no_callback_batch() {
    let group = RuntimeGroup::new(3, 3, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"batch_create": {
        "account_id": account_ids[2].clone()
        }, "id": 0 },
        {"action_function_call": {
        "promise_index": 0,
        "method_name": "call_promise",
        "arguments": [],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 0 }
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, ref0] );
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref1]);
    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
}

// single promise with callback (A->B=>C) with batch actions
#[test]
fn test_single_promise_with_callback_batch() {
    let group = RuntimeGroup::new(4, 4, near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"batch_create": {
            "account_id": account_ids[2].clone(),
        }, "id": 0 },
        {"action_function_call": {
            "promise_index": 0,
            "method_name": "call_promise",
            "arguments": [],
            "amount": "0",
            "gas": GAS_2,
        }, "id": 0 },
        {"batch_then": {
            "promise_index": 0,
            "account_id": account_ids[3].clone(),
        }, "id": 1},
        {"action_function_call": {
            "promise_index": 1,
            "method_name": "call_promise",
            "arguments": [],
            "amount": "0",
            "gas": GAS_2,
        }, "id": 1}
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, r2, ref0] );
    let data_id;
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, output_data_receivers, ..}), {
                        assert_eq!(output_data_receivers.len(), 1);
                        data_id = output_data_receivers[0].data_id;
                     },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref1]);
    assert_receipts!(group, account_ids[1].clone() => r2 @ account_ids[3].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, input_data_ids, ..}), {
                        assert_eq!(input_data_ids.len(), 1);
                        assert_eq!(data_id, input_data_ids[0].clone());
                     },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref2]);

    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
    assert_refund!(group, ref2 @ account_ids[0].clone());
}

#[test]
fn test_simple_transfer() {
    let group = RuntimeGroup::new(3, 3,near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"batch_create": {
            "account_id": account_ids[2].clone(),
        }, "id": 0 },
        {"action_transfer": {
            "promise_index": 0,
            "amount": "1000000000",
        }, "id": 0 }
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, ref0] );
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::Transfer(TransferAction{deposit}), {
                        assert_eq!(*deposit, 1000000000);
                     }
                     => [ref1] );

    assert_refund!(group, ref0 @ account_ids[0].clone());
    // For gas price difference
    assert_refund!(group, ref1 @ account_ids[0].clone());
}

#[test]
fn test_account_factory() {
    let group = RuntimeGroup::new(3, 2,near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"batch_create": {
            "account_id": account_ids[2].clone(),
        }, "id": 0 },
        {"action_create_account": {
            "promise_index": 0,
        }, "id": 0 },
        {"action_transfer": {
            "promise_index": 0,
            "amount": format!("{}", TESTING_INIT_BALANCE / 2),
        }, "id": 0 },
        {"action_deploy_contract": {
            "promise_index": 0,
            "code": base64::encode(near_test_contracts::rs_contract()),
        }, "id": 0 },
        {"action_function_call": {
            "promise_index": 0,
            "method_name": "call_promise",
            "arguments": [
                {"create": {
                "account_id": account_ids[0].clone(),
                "method_name": "call_promise",
                "arguments": [],
                "amount": "0",
                "gas": GAS_3,
                }, "id": 0}
            ],
            "amount": "0",
            "gas": GAS_2,
        }, "id": 0 },

        {"then": {
        "promise_index": 0,
        "account_id": account_ids[2].clone(),
        "method_name": "call_promise",
        "arguments": [
            {"create": {
            "account_id": account_ids[1].clone(),
            "method_name": "call_promise",
            "arguments": [],
            "amount": "0",
            "gas": GAS_3,
            }, "id": 0}
        ],
        "amount": "0",
        "gas": GAS_2,
        }, "id": 1}
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, r2, ref0] );

    let data_id;
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, output_data_receivers, ..}), {
                        assert_eq!(output_data_receivers.len(), 1);
                        data_id = output_data_receivers[0].data_id;
                        assert_eq!(output_data_receivers[0].receiver_id.as_ref(), account_ids[2].clone().as_ref());
                     },
                     actions,
                     a0, Action::CreateAccount(CreateAccountAction{}), {},
                     a1, Action::Transfer(TransferAction{deposit}), {
                        assert_eq!(*deposit, TESTING_INIT_BALANCE / 2);
                     },
                     a2, Action::DeployContract(DeployContractAction{code}), {
                        assert_eq!(code,near_test_contracts::rs_contract());
                     },
                     a3, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [r3, ref1] );
    assert_receipts!(group, account_ids[1].clone() => r2 @ account_ids[2].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, input_data_ids, ..}), {
                        assert_eq!(input_data_ids, &vec![data_id]);
                     },
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     }
                     => [r4, ref2] );
    assert_receipts!(group, account_ids[2].clone() => r3 @ account_ids[0].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_3);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref3] );
    assert_receipts!(group, account_ids[2].clone() => r4 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_3);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref4] );

    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
    assert_refund!(group, ref2 @ account_ids[0].clone());
    assert_refund!(group, ref3 @ account_ids[0].clone());
    assert_refund!(group, ref4 @ account_ids[0].clone());
}

#[test]
fn test_create_account_delete_account() {
    let group = RuntimeGroup::new(4, 3,near_test_contracts::rs_contract());
    let signer_sender = group.signers[0].clone();
    let account_ids = group.account_ids.clone();

    let data = serde_json::json!([
        {"batch_create": {
            "account_id": account_ids[3].clone(),
        }, "id": 0 },
        {"action_create_account": {
            "promise_index": 0,
        }, "id": 0 },
        {"action_transfer": {
            "promise_index": 0,
            "amount": format!("{}", TESTING_INIT_BALANCE / 2),
        }, "id": 0 },
        {"action_deploy_contract": {
            "promise_index": 0,
            "code": base64::encode(near_test_contracts::rs_contract()),
        }, "id": 0 },
        {"action_function_call": {
            "promise_index": 0,
            "method_name": "call_promise",
            "arguments": [
                {"create": {
                "account_id": account_ids[0].clone(),
                "method_name": "call_promise",
                "arguments": [],
                "amount": "0",
                "gas": GAS_3,
                }, "id": 0}
            ],
            "amount": "0",
            "gas": GAS_2,
        }, "id": 0 },
        {"action_delete_account": {
            "promise_index": 0,
            "beneficiary_id": account_ids[2].clone()
        }, "id": 0 },
    ]);

    let signed_transaction = SignedTransaction::from_actions(
        1,
        account_ids[0].clone(),
        account_ids[1].clone(),
        &signer_sender,
        vec![Action::FunctionCall(FunctionCallAction {
            method_name: "call_promise".to_string(),
            args: serde_json::to_vec(&data).unwrap(),
            gas: GAS_1,
            deposit: 0,
        })],
        CryptoHash::default(),
    );

    let handles = RuntimeGroup::start_runtimes(group.clone(), vec![signed_transaction.clone()]);
    for h in handles {
        h.join().unwrap();
    }

    assert_receipts!(group, signed_transaction => [r0]);
    assert_receipts!(group, account_ids[0].clone() => r0 @ account_ids[1].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_1);
                        assert_eq!(*deposit, 0);
                     }
                     => [r1, ref0] );
    assert_receipts!(group, account_ids[1].clone() => r1 @ account_ids[3].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::CreateAccount(CreateAccountAction{}), {},
                     a1, Action::Transfer(TransferAction{deposit}), {
                        assert_eq!(*deposit, TESTING_INIT_BALANCE / 2);
                     },
                     a2, Action::DeployContract(DeployContractAction{code}), {
                        assert_eq!(code,near_test_contracts::rs_contract());
                     },
                     a3, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_2);
                        assert_eq!(*deposit, 0);
                     },
                     a4, Action::DeleteAccount(DeleteAccountAction{beneficiary_id}), {
                        assert_eq!(beneficiary_id.as_ref(), account_ids[2].clone().as_ref());
                     }
                     => [r2, r3, ref1] );

    assert_receipts!(group, account_ids[3].clone() => r2 @ account_ids[0].clone(),
                     ReceiptEnum::Action(ActionReceipt{actions, ..}), {},
                     actions,
                     a0, Action::FunctionCall(FunctionCallAction{gas, deposit, ..}), {
                        assert_eq!(*gas, GAS_3);
                        assert_eq!(*deposit, 0);
                     }
                     => [ref2] );
    assert_refund!(group, r3 @ account_ids[2].clone());

    assert_refund!(group, ref0 @ account_ids[0].clone());
    assert_refund!(group, ref1 @ account_ids[0].clone());
    assert_refund!(group, ref2 @ account_ids[0].clone());
}
