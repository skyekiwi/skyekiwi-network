use skw_vm_primitives::{
    contract_runtime::{Balance, AccountId},
    config::{VMLimitConfig, RuntimeConfig},
    receipt::{ActionReceipt, DataReceipt, Receipt, ReceiptEnum},
    transaction::{
        Action, DeployContractAction, FunctionCallAction, SignedTransaction,
    },
    errors::{
        ActionsValidationError, InvalidTxError, ReceiptValidationError,
        RuntimeError,
    },
};

use skw_vm_store::{
    get_account, set_account, TrieUpdate,
};

use crate::config::{tx_cost, TransactionCost, total_prepaid_gas};
use crate::VerificationResult;

/// Validates the transaction without using the state. It allows any node to validate a
/// transaction before forwarding it to the node that tracks the `signer_id` account.
pub fn validate_transaction(
    config: &RuntimeConfig,
    gas_price: Balance,
    signed_transaction: &SignedTransaction,
    verify_signature: bool,
) -> Result<TransactionCost, RuntimeError> {
    let transaction = &signed_transaction.transaction;
    let signer_id = &transaction.signer_id;

    if verify_signature
        && !signed_transaction
            .signature
            .verify(signed_transaction.get_hash().as_ref(), &transaction.signer_id.as_ref())
    {
        return Err(InvalidTxError::InvalidSignature.into());
    }

    let transaction_size = signed_transaction.get_size();
    let max_transaction_size = config.wasm_config.limit_config.max_transaction_size;
    if transaction_size > max_transaction_size {
        return Err(InvalidTxError::TransactionSizeExceeded {
            size: transaction_size,
            limit: max_transaction_size,
        }
        .into());
    }

    validate_actions(&config.wasm_config.limit_config, &transaction.actions)
        .map_err(InvalidTxError::ActionsValidation)?;

    let sender_is_receiver = &transaction.receiver_id == signer_id;

    tx_cost(
        &config.transaction_costs,
        transaction,
        gas_price,
        sender_is_receiver,
    )
    .map_err(|_| InvalidTxError::CostOverflow.into())
}

/// Verifies the signed transaction on top of given state, charges transaction fees
/// and balances, and updates the state for the used account and access keys.
pub fn verify_and_charge_transaction(
    config: &RuntimeConfig,
    state_update: &mut TrieUpdate,
    gas_price: Balance,
    signed_transaction: &SignedTransaction,
    verify_signature: bool,
) -> Result<VerificationResult, RuntimeError> {
    let TransactionCost { gas_burnt, gas_remaining, receipt_gas_price, total_cost, burnt_amount } =
        validate_transaction(
            config,
            gas_price,
            signed_transaction,
            verify_signature,
        )?;
    let transaction = &signed_transaction.transaction;
    let signer_id = &transaction.signer_id;

    let mut signer = match get_account(state_update, signer_id)? {
        Some(signer) => signer,
        None => {
            return Err(InvalidTxError::SignerDoesNotExist { signer_id: signer_id.clone() }.into());
        }
    };

    if transaction.nonce <= signer.nonce {
        return Err(InvalidTxError::InvalidNonce {
            tx_nonce: transaction.nonce,
            ak_nonce: signer.nonce,
        }
        .into());
    }

    signer.nonce = transaction.nonce;

    signer.set_amount(signer.amount().checked_sub(total_cost).ok_or_else(|| {
        InvalidTxError::NotEnoughBalance {
            signer_id: signer_id.clone(),
            balance: signer.amount(),
            cost: total_cost,
        }
    })?);

    set_account(state_update, signer_id.clone(), &signer);

    Ok(VerificationResult { gas_burnt, gas_remaining, receipt_gas_price, burnt_amount })
}

/// Validates a given receipt. Checks validity of the Action or Data receipt.
pub(crate) fn validate_receipt(
    limit_config: &VMLimitConfig,
    receipt: &Receipt,
) -> Result<(), ReceiptValidationError> {
    
    // TODO: have these removed or changed. We use a different account sys now
    // We retain these checks here as to maintain backwards compatibility
    // with AccountId validation since we illegally parse an AccountId
    // in near-vm-logic/logic.rs#fn(VMLogic::read_and_parse_account_id)
    AccountId::validate(&receipt.predecessor_id).map_err(|_| {
        ReceiptValidationError::InvalidPredecessorId {
            account_id: receipt.predecessor_id.clone(),
        }
    })?;
    AccountId::validate(&receipt.receiver_id).map_err(|_| {
        ReceiptValidationError::InvalidReceiverId { account_id: receipt.receiver_id.clone() }
    })?;

    match &receipt.receipt {
        ReceiptEnum::Action(action_receipt) => {
            validate_action_receipt(limit_config, action_receipt)
        }
        ReceiptEnum::Data(data_receipt) => validate_data_receipt(limit_config, data_receipt),
    }
}

/// Validates given ActionReceipt. Checks validity of the number of input data dependencies and all actions.
fn validate_action_receipt(
    limit_config: &VMLimitConfig,
    receipt: &ActionReceipt,
) -> Result<(), ReceiptValidationError> {
    if receipt.input_data_ids.len() as u64 > limit_config.max_number_input_data_dependencies {
        return Err(ReceiptValidationError::NumberInputDataDependenciesExceeded {
            number_of_input_data_dependencies: receipt.input_data_ids.len() as u64,
            limit: limit_config.max_number_input_data_dependencies,
        });
    }
    validate_actions(limit_config, &receipt.actions)
        .map_err(ReceiptValidationError::ActionsValidation)
}

/// Validates given data receipt. Checks validity of the length of the returned data.
fn validate_data_receipt(
    limit_config: &VMLimitConfig,
    receipt: &DataReceipt,
) -> Result<(), ReceiptValidationError> {
    let data_len = receipt.data.as_ref().map(|data| data.len()).unwrap_or(0);
    if data_len as u64 > limit_config.max_length_returned_data {
        return Err(ReceiptValidationError::ReturnedValueLengthExceeded {
            length: data_len as u64,
            limit: limit_config.max_length_returned_data,
        });
    }
    Ok(())
}

/// Validates given actions:
///
/// - Checks limits if applicable.
/// - Checks that the total number of actions doesn't exceed the limit.
/// - Validates each individual action.
/// - Checks that the total prepaid gas doesn't exceed the limit.
pub(crate) fn validate_actions(
    limit_config: &VMLimitConfig,
    actions: &[Action],
) -> Result<(), ActionsValidationError> {
    if actions.len() as u64 > limit_config.max_actions_per_receipt {
        return Err(ActionsValidationError::TotalNumberOfActionsExceeded {
            total_number_of_actions: actions.len() as u64,
            limit: limit_config.max_actions_per_receipt,
        });
    }

    let mut iter = actions.iter().peekable();
    while let Some(action) = iter.next() {
        if let Action::DeleteAccount(_) = action {
            if iter.peek().is_some() {
                return Err(ActionsValidationError::DeleteActionMustBeFinal);
            }
        }
        validate_action(limit_config, action)?;
    }

    let total_prepaid_gas =
        total_prepaid_gas(actions).map_err(|_| ActionsValidationError::IntegerOverflow)?;
    if total_prepaid_gas > limit_config.max_total_prepaid_gas {
        return Err(ActionsValidationError::TotalPrepaidGasExceeded {
            total_prepaid_gas,
            limit: limit_config.max_total_prepaid_gas,
        });
    }

    Ok(())
}

/// Validates a single given action. Checks limits if applicable.
pub fn validate_action(
    limit_config: &VMLimitConfig,
    action: &Action,
) -> Result<(), ActionsValidationError> {
    match action {
        Action::CreateAccount(_) => Ok(()),
        Action::DeployContract(a) => validate_deploy_contract_action(limit_config, a),
        Action::FunctionCall(a) => validate_function_call_action(limit_config, a),
        Action::Transfer(_) => Ok(()),
        Action::DeleteAccount(_) => Ok(()),
    }
}

/// Validates `DeployContractAction`. Checks that the given contract size doesn't exceed the limit.
fn validate_deploy_contract_action(
    limit_config: &VMLimitConfig,
    action: &DeployContractAction,
) -> Result<(), ActionsValidationError> {
    if action.code.len() as u64 > limit_config.max_contract_size {
        return Err(ActionsValidationError::ContractSizeExceeded {
            size: action.code.len() as u64,
            limit: limit_config.max_contract_size,
        });
    }

    Ok(())
}

/// Validates `FunctionCallAction`. Checks that the method name length doesn't exceed the limit and
/// the length of the arguments doesn't exceed the limit.
fn validate_function_call_action(
    limit_config: &VMLimitConfig,
    action: &FunctionCallAction,
) -> Result<(), ActionsValidationError> {
    if action.gas == 0 {
        return Err(ActionsValidationError::FunctionCallZeroAttachedGas);
    }

    if action.method_name.len() as u64 > limit_config.max_length_method_name {
        return Err(ActionsValidationError::FunctionCallMethodNameLengthExceeded {
            length: action.method_name.len() as u64,
            limit: limit_config.max_length_method_name,
        });
    }

    if action.args.len() as u64 > limit_config.max_arguments_length {
        return Err(ActionsValidationError::FunctionCallArgumentsLengthExceeded {
            length: action.args.len() as u64,
            limit: limit_config.max_arguments_length,
        });
    }

    Ok(())
}

//     Ok(())
// }
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use skw_vm_primitives::crypto::{InMemorySigner, KeyType, Signer};
    use skw_vm_primitives::test_utils::account_new;
    use skw_vm_primitives::transaction::{
        CreateAccountAction, DeleteAccountAction, TransferAction,
    };
    use skw_vm_primitives::contract_runtime::{
        AccountId, Balance, MerkleHash, StateChangeCause, CryptoHash, Nonce,
    };
    use skw_vm_store::test_utils::create_tries;

    pub fn alice_account() -> AccountId {
        AccountId::test()
    }
    pub fn bob_account() -> AccountId {
        AccountId::test2()
    }

    use super::*;

    /// Initial balance used in tests.
    const TESTING_INIT_BALANCE: Balance = 1_000_000_000 * NEAR_BASE;

    /// One NEAR, divisible by 10^24.
    const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

    fn setup_common(
        seed: CryptoHash,
        initial_balance: Balance,
        initial_locked: Balance,
        initial_nonce: Nonce,
    ) -> (Arc<InMemorySigner>, AccountId, TrieUpdate, Balance) {
        let tries = create_tries();
        let root = MerkleHash::default();

        let signer = Arc::new(InMemorySigner::from_seed(
            KeyType::SR25519, &seed
        ));

        let account_id: AccountId = AccountId::new( signer.public_key() );
        let mut initial_state = tries.new_trie_update(root);
        
        let mut initial_account = account_new(initial_balance, CryptoHash::default(), initial_nonce);
        initial_account.set_locked(initial_locked);
        
        set_account(&mut initial_state, account_id.clone(), &initial_account);

        initial_state.commit(StateChangeCause::InitialState);
        let trie_changes = initial_state.finalize().unwrap().0;
        let (store_update, root) =
            tries.apply_all(&trie_changes).unwrap();
        store_update.commit().unwrap();

        (signer, account_id, tries.new_trie_update(root), 100)
    }

    // Transactions
    #[test]
    fn test_validate_transaction_valid() {
        let config = RuntimeConfig::test();
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 0);

        let deposit = 100;
        let transaction = SignedTransaction::send_money(
            1,
            account_id.clone(),
            alice_account(),
            &*signer,
            deposit,
            CryptoHash::default(),
        );
        validate_transaction(&config, gas_price, &transaction, true)
            .expect("valid transaction");
        let verification_result = verify_and_charge_transaction(
            &config,
            &mut state_update,
            gas_price,
            &transaction,
            true,
        )
        .expect("valid transaction");
        // Should not be free. Burning for sending
        assert!(verification_result.gas_burnt > 0);
        // All burned gas goes to the validators at current gas price
        assert_eq!(
            verification_result.burnt_amount,
            Balance::from(verification_result.gas_burnt) * gas_price
        );

        let account = get_account(&state_update, &account_id).unwrap().unwrap();
        // Balance is decreased by the (TX fees + transfer balance).
        assert_eq!(
            account.amount(),
            TESTING_INIT_BALANCE
                - Balance::from(verification_result.gas_remaining)
                    * verification_result.receipt_gas_price
                - verification_result.burnt_amount
                - deposit
        );
        assert_eq!(account.nonce, 1);
    }

    #[test]
    fn test_validate_transaction_invalid_signature() {
        let config = RuntimeConfig::test();
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 0);

        let mut tx = SignedTransaction::send_money(
            1,
            account_id,
            bob_account(),
            &*signer,
            100,
            CryptoHash::default(),
        );
        tx.signature = signer.sign(CryptoHash::default().as_ref());

        assert_eq!(
            verify_and_charge_transaction(
                &config,
                &mut state_update,
                gas_price,
                &tx,
                true,
            )
            .expect_err("expected an error"),
            RuntimeError::InvalidTxError(InvalidTxError::InvalidSignature),
        );
    }

    #[test]
    fn test_validate_transaction_invalid_bad_action() {
        let mut config = RuntimeConfig::test();
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 0);

        config.wasm_config.limit_config.max_total_prepaid_gas = 100;

        assert_eq!(
            verify_and_charge_transaction(
                &config,
                &mut state_update,
                gas_price,
                &SignedTransaction::from_actions(
                    1,
                    account_id,
                    bob_account(),
                    &*signer,
                    vec![Action::FunctionCall(FunctionCallAction {
                        method_name: "hello".to_string(),
                        args: b"abc".to_vec(),
                        gas: 200,
                        deposit: 0,
                    })],
                    CryptoHash::default(),
                ),
                true,
            )
            .expect_err("expected an error"),
            RuntimeError::InvalidTxError(InvalidTxError::ActionsValidation(
                ActionsValidationError::TotalPrepaidGasExceeded {
                    total_prepaid_gas: 200,
                    limit: 100,
                },
            )),
        );

        // assert_err_both_validations(
        //     &config,
        //     &mut state_update,
        //     gas_price,
        //     &SignedTransaction::from_actions(
        //         1,
        //         account_id,
        //         bob_account(),
        //         &*signer,
        //         vec![Action::FunctionCall(FunctionCallAction {
        //             method_name: "hello".to_string(),
        //             args: b"abc".to_vec(),
        //             gas: 200,
        //             deposit: 0,
        //         })],
        //         CryptoHash::default(),
        //     ),
        //     ,
        // );
    }

    #[test]
    fn test_validate_transaction_invalid_bad_nonce() {
        let config = RuntimeConfig::test();
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 2);

        assert_eq!(
            verify_and_charge_transaction(
                &config,
                &mut state_update,
                gas_price,
                &SignedTransaction::send_money(
                    1,
                    account_id,
                    bob_account(),
                    &*signer,
                    100,
                    CryptoHash::default(),
                ),
                true,
            )
            .expect_err("expected an error"),
            RuntimeError::InvalidTxError(InvalidTxError::InvalidNonce { tx_nonce: 1, ak_nonce: 2 }),
        );
    }

    #[test]
    fn test_validate_transaction_invalid_balance_overflow() {
        let config = RuntimeConfig::test();
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 2);

        assert_eq!(
            verify_and_charge_transaction(
                &config,
                &mut state_update,
                gas_price,
                &SignedTransaction::send_money(
                    1,
                    account_id,
                    bob_account(),
                    &*signer,
                    u128::max_value(),
                    CryptoHash::default(),
                ),
                true,
            )
            .expect_err("expected an error"),
            RuntimeError::InvalidTxError(InvalidTxError::CostOverflow),
        );
    }

    #[test]
    fn test_validate_transaction_invalid_not_enough_balance() {
        let config = RuntimeConfig::test();
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 0);

        let err = verify_and_charge_transaction(
            &config,
            &mut state_update,
            gas_price,
            &SignedTransaction::send_money(
                1,
                account_id.clone(),
                bob_account(),
                &*signer,
                TESTING_INIT_BALANCE,
                CryptoHash::default(),
            ),
            true,
        )
        .expect_err("expected an error");
        if let RuntimeError::InvalidTxError(InvalidTxError::NotEnoughBalance {
            signer_id,
            balance,
            cost,
        }) = err
        {
            assert_eq!(signer_id, account_id);
            assert_eq!(balance, TESTING_INIT_BALANCE);
            assert!(cost > balance);
        } else {
            panic!("Incorrect error");
        }
    }

    /// Setup: account has 1B yoctoN and is 180 bytes. Storage requirement is 1M per byte.
    /// Test that such account can not send 950M yoctoN out as that will leave it under storage requirements.
    // TODO: we have removed storage stake - 
    #[test]
    fn test_validate_transaction_invalid_low_balance() {
        let mut config = RuntimeConfig::test();
        config.storage_amount_per_byte = 10_000_000;
        let initial_balance = 1_000_000_000;
        let transfer_amount = 800_000_000;
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],initial_balance, 0, 0);

        assert_eq!(
            verify_and_charge_transaction(
                &config,
                &mut state_update,
                gas_price,
                &SignedTransaction::send_money(
                    1,
                    account_id.clone(),
                    bob_account(),
                    &*signer,
                    transfer_amount,
                    CryptoHash::default(),
                ),
                true,
            )
            .expect_err("expected an error"),
            RuntimeError::InvalidTxError(InvalidTxError::NotEnoughBalance { 
                signer_id: account_id, 
                balance: 1000000000, 
                cost: 45306860187500 
            })
        );
    }

    #[test]
    fn test_validate_transaction_exceeding_tx_size_limit() {
        let (signer, account_id, mut state_update, gas_price) =
            setup_common([0; 32],TESTING_INIT_BALANCE, 0, 0);

        let transaction = SignedTransaction::from_actions(
            1,
            account_id,
            bob_account(),
            &*signer,
            vec![Action::DeployContract(DeployContractAction { code: vec![1; 5] })],
            CryptoHash::default(),
        );
        let transaction_size = transaction.get_size();

        let mut config = RuntimeConfig::test();
        let max_transaction_size = transaction_size - 1;
        config.wasm_config.limit_config.max_transaction_size = transaction_size - 1;

        assert_eq!(
            verify_and_charge_transaction(
                &config,
                &mut state_update,
                gas_price,
                &transaction,
                false,
            )
            .expect_err("expected an error"),
            RuntimeError::InvalidTxError(InvalidTxError::TransactionSizeExceeded {
                size: transaction_size,
                limit: max_transaction_size
            }),
        );

        config.wasm_config.limit_config.max_transaction_size = transaction_size + 1;
        verify_and_charge_transaction(
            &config,
            &mut state_update,
            gas_price,
            &transaction,
            false,
        )
        .expect("valid transaction");
    }

    // Receipts
    #[test]
    fn test_validate_receipt_valid() {
        let limit_config = VMLimitConfig::test();
        validate_receipt(&limit_config, 
            &Receipt {
                predecessor_id: AccountId::system(), 
                receiver_id: AccountId::test(),
                receipt_id: CryptoHash::default(),
    
                receipt: ReceiptEnum::Action(ActionReceipt {
                    signer_id: AccountId::system(),
                    gas_price: 0,
                    output_data_receivers: vec![],
                    input_data_ids: vec![],
                    actions: vec![Action::Transfer(TransferAction { deposit: 1 })],
                }),
            })
            .expect("valid receipt");
    }

    #[test]
    fn test_validate_action_receipt_too_many_input_deps() {
        let mut limit_config = VMLimitConfig::test();
        limit_config.max_number_input_data_dependencies = 1;
        assert_eq!(
            validate_action_receipt(
                &limit_config,
                &ActionReceipt {
                    signer_id: alice_account(),
                    gas_price: 100,
                    output_data_receivers: vec![],
                    input_data_ids: vec![CryptoHash::default(), CryptoHash::default()],
                    actions: vec![]
                }
            )
            .expect_err("expected an error"),
            ReceiptValidationError::NumberInputDataDependenciesExceeded {
                number_of_input_data_dependencies: 2,
                limit: 1
            }
        );
    }

    // DataReceipt

    #[test]
    fn test_validate_data_receipt_valid() {
        let limit_config = VMLimitConfig::test();
        validate_data_receipt(
            &limit_config,
            &DataReceipt { data_id: CryptoHash::default(), data: None },
        )
        .expect("valid data receipt");
        let data = b"hello".to_vec();
        validate_data_receipt(
            &limit_config,
            &DataReceipt { data_id: CryptoHash::default(), data: Some(data) },
        )
        .expect("valid data receipt");
    }

    #[test]
    fn test_validate_data_receipt_too_much_data() {
        let mut limit_config = VMLimitConfig::test();
        let data = b"hello".to_vec();
        limit_config.max_length_returned_data = data.len() as u64 - 1;
        assert_eq!(
            validate_data_receipt(
                &limit_config,
                &DataReceipt { data_id: CryptoHash::default(), data: Some(data.clone()) }
            )
            .expect_err("expected an error"),
            ReceiptValidationError::ReturnedValueLengthExceeded {
                length: data.len() as u64,
                limit: limit_config.max_length_returned_data
            }
        );
    }

    // Group of actions
    #[test]
    fn test_validate_actions_empty() {
        let limit_config = VMLimitConfig::test();
        validate_actions(&limit_config, &[]).expect("empty actions");
    }

    #[test]
    fn test_validate_actions_valid_function_call() {
        let limit_config = VMLimitConfig::test();
        validate_actions(
            &limit_config,
            &vec![Action::FunctionCall(FunctionCallAction {
                method_name: "hello".to_string(),
                args: b"abc".to_vec(),
                gas: 100,
                deposit: 0,
            })],
        )
        .expect("valid function call action");
    }

    #[test]
    fn test_validate_actions_too_much_gas() {
        let mut limit_config = VMLimitConfig::test();
        limit_config.max_total_prepaid_gas = 220;
        assert_eq!(
            validate_actions(
                &limit_config,
                &vec![
                    Action::FunctionCall(FunctionCallAction {
                        method_name: "hello".to_string(),
                        args: b"abc".to_vec(),
                        gas: 100,
                        deposit: 0,
                    }),
                    Action::FunctionCall(FunctionCallAction {
                        method_name: "hello".to_string(),
                        args: b"abc".to_vec(),
                        gas: 150,
                        deposit: 0,
                    })
                ]
            )
            .expect_err("expected an error"),
            ActionsValidationError::TotalPrepaidGasExceeded { total_prepaid_gas: 250, limit: 220 }
        );
    }

    #[test]
    fn test_validate_actions_gas_overflow() {
        let mut limit_config = VMLimitConfig::test();
        limit_config.max_total_prepaid_gas = 220;
        assert_eq!(
            validate_actions(
                &limit_config,
                &vec![
                    Action::FunctionCall(FunctionCallAction {
                        method_name: "hello".to_string(),
                        args: b"abc".to_vec(),
                        gas: u64::max_value() / 2 + 1,
                        deposit: 0,
                    }),
                    Action::FunctionCall(FunctionCallAction {
                        method_name: "hello".to_string(),
                        args: b"abc".to_vec(),
                        gas: u64::max_value() / 2 + 1,
                        deposit: 0,
                    })
                ]
            )
            .expect_err("Expected an error"),
            ActionsValidationError::IntegerOverflow,
        );
    }

    #[test]
    fn test_validate_actions_num_actions() {
        let mut limit_config = VMLimitConfig::test();
        limit_config.max_actions_per_receipt = 1;
        assert_eq!(
            validate_actions(
                &limit_config,
                &vec![
                    Action::CreateAccount(CreateAccountAction {}),
                    Action::CreateAccount(CreateAccountAction {}),
                ]
            )
            .expect_err("Expected an error"),
            ActionsValidationError::TotalNumberOfActionsExceeded {
                total_number_of_actions: 2,
                limit: 1
            },
        );
    }

    #[test]
    fn test_validate_delete_must_be_final() {
        let mut limit_config = VMLimitConfig::test();
        limit_config.max_actions_per_receipt = 3;
        assert_eq!(
            validate_actions(
                &limit_config,
                &vec![
                    Action::DeleteAccount(DeleteAccountAction {
                        beneficiary_id: AccountId::test2()
                    }),
                    Action::CreateAccount(CreateAccountAction {}),
                ]
            )
            .expect_err("Expected an error"),
            ActionsValidationError::DeleteActionMustBeFinal,
        );
    }

    #[test]
    fn test_validate_delete_must_work_if_its_final() {
        let mut limit_config = VMLimitConfig::test();
        limit_config.max_actions_per_receipt = 3;
        assert_eq!(
            validate_actions(
                &limit_config,
                &vec![
                    Action::CreateAccount(CreateAccountAction {}),
                    Action::DeleteAccount(DeleteAccountAction {
                        beneficiary_id: AccountId::test2()
                    }),
                ]
            ),
            Ok(()),
        );
    }

    // Individual actions

    #[test]
    fn test_validate_action_valid_create_account() {
        validate_action(&VMLimitConfig::test(), &Action::CreateAccount(CreateAccountAction {}))
            .expect("valid action");
    }

    #[test]
    fn test_validate_action_valid_function_call() {
        validate_action(
            &VMLimitConfig::test(),
            &Action::FunctionCall(FunctionCallAction {
                method_name: "hello".to_string(),
                args: b"abc".to_vec(),
                gas: 100,
                deposit: 0,
            }),
        )
        .expect("valid action");
    }

    #[test]
    fn test_validate_action_invalid_function_call_zero_gas() {
        assert_eq!(
            validate_action(
                &VMLimitConfig::test(),
                &Action::FunctionCall(FunctionCallAction {
                    method_name: "new".to_string(),
                    args: vec![],
                    gas: 0,
                    deposit: 0,
                }),
            )
            .expect_err("expected an error"),
            ActionsValidationError::FunctionCallZeroAttachedGas,
        );
    }

    #[test]
    fn test_validate_action_valid_transfer() {
        validate_action(&VMLimitConfig::test(), &Action::Transfer(TransferAction { deposit: 10 }))
            .expect("valid action");
    }

    #[test]
    fn test_validate_action_valid_delete_account() {
        validate_action(
            &VMLimitConfig::test(),
            &Action::DeleteAccount(DeleteAccountAction { beneficiary_id: alice_account() }),
        )
        .expect("valid action");
    }
}
