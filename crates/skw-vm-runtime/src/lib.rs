use std::collections::{HashMap};
use std::rc::Rc;

use log::debug;

use borsh::BorshSerialize;
pub use skw_vm_primitives::crypto;

use skw_vm_primitives::profile::ProfileData;
pub use skw_vm_primitives::apply_state::ApplyState;
use skw_vm_primitives::fees::RuntimeFeesConfig;

use skw_vm_primitives::{
    errors::{ActionError, ActionErrorKind, RuntimeError, TxExecutionError},
    contract_runtime::{CryptoHash, AccountId, Balance, Gas, StateChangeCause, StateRoot, RawStateChangesWithTrieKey, ContractCode},
    receipt::{
        ActionReceipt, DataReceipt, DelayedReceiptIndices, Receipt, ReceiptEnum, ReceivedData,
    },
    account::{Account, AccessKey},
    state_record::StateRecord,
    trie_key::TrieKey,
    utils::{
        create_action_hash, create_receipt_id_from_receipt, create_receipt_id_from_transaction,
    },
    crypto::PublicKey,
};
use skw_vm_primitives::transaction::{
    Action, ExecutionOutcome, ExecutionOutcomeWithId, ExecutionStatus, LogEntry,
    SignedTransaction,
};
use skw_vm_store::{
    get, get_account, get_postponed_receipt, get_received_data, remove_postponed_receipt, set,
    set_account, set_postponed_receipt, set_received_data,set_code, set_access_key, PartialStorage, ShardTries,
    StorageError, Trie, TrieChanges, TrieUpdate, StoreUpdate,
};

use skw_vm_host::types::PromiseResult;
use skw_vm_host::ReturnData;

pub use skw_vm_engine::with_ext_cost_counter;
mod balance_checker;

use crate::actions::*;
use crate::balance_checker::check_balance;
use crate::config::{
    exec_fee, safe_add_balance, safe_add_gas, safe_gas_to_balance, total_deposit,
    total_prepaid_exec_fees, total_prepaid_gas, RuntimeConfig,
};
use crate::verifier::validate_receipt;
pub use crate::verifier::{validate_transaction, verify_and_charge_transaction};

mod actions;
pub mod cache;
pub mod config;
pub mod ext;
pub mod state_viewer;
mod verifier;

const EXPECT_ACCOUNT_EXISTS: &str = "account exists, checked above";

#[derive(Debug)]
pub struct VerificationResult {
    /// The amount gas that was burnt to convert the transaction into a receipt and send it.
    pub gas_burnt: Gas,
    /// The remaining amount of gas in the receipt.
    pub gas_remaining: Gas,
    /// The gas price at which the gas was purchased in the receipt.
    pub receipt_gas_price: Balance,
    /// The balance that was burnt to convert the transaction into a receipt and send it.
    pub burnt_amount: Balance,
}

#[derive(Debug, Default)]
pub struct ApplyStats {
    pub tx_burnt_amount: Balance,
    pub slashed_burnt_amount: Balance,
    pub other_burnt_amount: Balance,
    /// This is a negative amount. This amount was not charged from the account that issued
    /// the transaction. It's likely due to the delayed queue of the receipts.
    pub gas_deficit_amount: Balance,
}

pub struct ApplyResult {
    pub state_root: StateRoot,
    pub trie_changes: TrieChanges,
    pub outgoing_receipts: Vec<Receipt>,
    pub outcomes: Vec<ExecutionOutcomeWithId>,
    pub state_changes: Vec<RawStateChangesWithTrieKey>,
    pub stats: ApplyStats,
    pub processed_delayed_receipts: Vec<Receipt>,
    pub proof: Option<PartialStorage>,
}

#[derive(Debug)]
pub struct ActionResult {
    pub gas_burnt: Gas,
    pub gas_burnt_for_function_call: Gas,
    pub gas_used: Gas,
    pub result: Result<ReturnData, ActionError>,
    pub logs: Vec<LogEntry>,
    pub new_receipts: Vec<Receipt>,
    pub profile: ProfileData,
}

impl ActionResult {
    pub fn merge(&mut self, mut next_result: ActionResult) -> Result<(), RuntimeError> {
        assert!(next_result.gas_burnt_for_function_call <= next_result.gas_burnt);
        assert!(
            next_result.gas_burnt <= next_result.gas_used,
            "Gas burnt {} <= Gas used {}",
            next_result.gas_burnt,
            next_result.gas_used
        );

        self.gas_burnt = safe_add_gas(self.gas_burnt, next_result.gas_burnt)?;
        self.gas_burnt_for_function_call = safe_add_gas(
            self.gas_burnt_for_function_call,
            next_result.gas_burnt_for_function_call,
        )?;
        self.gas_used = safe_add_gas(self.gas_used, next_result.gas_used)?;
        self.profile.merge(&next_result.profile);
        self.result = next_result.result;
        self.logs.append(&mut next_result.logs);
        if let Ok(ReturnData::ReceiptIndex(ref mut receipt_index)) = self.result {
            // Shifting local receipt index to be global receipt index.
            *receipt_index += self.new_receipts.len() as u64;
        }
        if self.result.is_ok() {
            self.new_receipts.append(&mut next_result.new_receipts);
        } else {
            self.new_receipts.clear();
        }
        Ok(())
    }
}

impl Default for ActionResult {
    fn default() -> Self {
        Self {
            gas_burnt: 0,
            gas_burnt_for_function_call: 0,
            gas_used: 0,
            result: Ok(ReturnData::None),
            logs: vec![],
            new_receipts: vec![],
            profile: Default::default(),
        }
    }
}

pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    // TODO: mark it that these are events
    fn print_log(log: &[LogEntry]) {
        if log.is_empty() {
            return;
        }
        let log_str = log.iter().fold(String::new(), |acc, s| {
            if acc.is_empty() {
                s.to_string()
            } else {
                acc + "\n" + s
            }
        });
        debug!(target: "runtime", "{}", log_str);
    }

    /// Takes one signed transaction, verifies it and converts it to a receipt. Add this receipt
    /// either to the new local receipts if the signer is the same as receiver or to the new
    /// outgoing receipts.
    /// When transaction is converted to a receipt, the account is charged for the full value of
    /// the generated receipt.
    /// In case of successful verification (expected for valid chunks), returns the receipt and
    /// `ExecutionOutcomeWithId` for the transaction.
    /// In case of an error, returns either `InvalidTxError` if the transaction verification failed
    /// or a `StorageError` wrapped into `RuntimeError`.
    fn process_transaction(
        &self,
        state_update: &mut TrieUpdate,
        apply_state: &ApplyState,
        signed_transaction: &SignedTransaction,
        stats: &mut ApplyStats,
    ) -> Result<(Receipt, ExecutionOutcomeWithId), RuntimeError> {
        let _span =
            tracing::debug_span!(target: "runtime", "Runtime::process_transaction").entered();
        // metrics::TRANSACTION_PROCESSED_TOTAL.inc();

        match verify_and_charge_transaction(
            &apply_state.config,
            state_update,
            apply_state.gas_price,
            signed_transaction,
            true,
        ) {
            Ok(verification_result) => {
                // metrics::TRANSACTION_PROCESSED_SUCCESSFULLY_TOTAL.inc();
                state_update.commit(StateChangeCause::TransactionProcessing {
                    tx_hash: signed_transaction.get_hash(),
                });
                let transaction = &signed_transaction.transaction;
                let receipt_id = create_receipt_id_from_transaction(signed_transaction);
                let receipt = Receipt {
                    predecessor_id: transaction.signer_id.clone(),
                    receiver_id: transaction.receiver_id.clone(),
                    receipt_id,
                    receipt: ReceiptEnum::Action(ActionReceipt {
                        signer_id: transaction.signer_id.clone(),
                        signer_public_key: transaction.public_key.clone(),
                        gas_price: verification_result.receipt_gas_price,
                        output_data_receivers: vec![],
                        input_data_ids: vec![],
                        actions: transaction.actions.clone(),
                    }),
                };
                stats.tx_burnt_amount =
                    safe_add_balance(stats.tx_burnt_amount, verification_result.burnt_amount)?;
                let outcome = ExecutionOutcomeWithId {
                    id: signed_transaction.get_hash(),
                    outcome: ExecutionOutcome {
                        status: ExecutionStatus::SuccessReceiptId(receipt.receipt_id),
                        logs: vec![],
                        receipt_ids: vec![receipt.receipt_id],
                        gas_burnt: verification_result.gas_burnt,
                        tokens_burnt: verification_result.burnt_amount,
                        executor_id: transaction.signer_id.clone(),
                        // TODO: profile data is only counted in apply_action, which only happened at process_receipt
                        // VerificationResult needs updates to incorporate profile data to support profile data of txns
                        // metadata: ExecutionMetadata::V1,
                        // DIFF: added ProfileData in sync with skw-vm-primitives
                        profile_data: None,
                    },
                };
                Ok((receipt, outcome))
            }
            Err(e) => {
                // metrics::TRANSACTION_PROCESSED_FAILED_TOTAL.inc();
                state_update.rollback();
                Err(e)
            }
        }
    }

    fn apply_action(
        &self,
        action: &Action,
        state_update: &mut TrieUpdate,
        apply_state: &ApplyState,
        account: &mut Option<Account>,
        actor_id: &mut AccountId,
        receipt: &Receipt,
        action_receipt: &ActionReceipt,
        promise_results: &[PromiseResult],
        action_hash: &CryptoHash,
        action_index: usize,
        actions: &[Action],
    ) -> Result<ActionResult, RuntimeError> {
        // println!("enter apply_action");
        let mut result = ActionResult::default();
        let exec_fees = exec_fee(
            &apply_state.config.transaction_costs,
            action,
            &receipt.receiver_id,
        );
        result.gas_burnt += exec_fees;
        result.gas_used += exec_fees;
        let account_id = &receipt.receiver_id;
        let is_the_only_action = actions.len() == 1;
        let is_refund = AccountId::is_system(&receipt.predecessor_id);
        
        // Account validation
        if let Err(e) = check_account_existence(
            action,
            account,
            account_id,
            is_the_only_action,
            is_refund,
        ) {
            result.result = Err(e);
            return Ok(result);
        }
        // Permission validation
        if let Err(e) = check_actor_permissions(action, account, actor_id, account_id) {
            result.result = Err(e);
            return Ok(result);
        }

        match action {
            Action::CreateAccount(_) => {
                // metrics::ACTION_CREATE_ACCOUNT_TOTAL.inc();
                action_create_account(
                    &apply_state.config.transaction_costs,
                    &apply_state.config.account_creation_config,
                    account,
                    actor_id,
                    &receipt.receiver_id,
                    &receipt.predecessor_id,
                    &mut result,
                );
            }
            Action::DeployContract(deploy_contract) => {
                // metrics::ACTION_DEPLOY_CONTRACT_TOTAL.inc();
                action_deploy_contract(
                    state_update,
                    account_id,
                    account.as_mut().expect(EXPECT_ACCOUNT_EXISTS),
                    deploy_contract,
                    apply_state,
                )?;
            }
            Action::Transfer(transfer) => {
                // metrics::ACTION_TRANSFER_TOTAL.inc();
                if let Some(account) = account.as_mut() {
                    action_transfer(account, transfer)?;
                    // Check if this is a gas refund, then try to refund the access key allowance.
                    if is_refund && action_receipt.signer_id == receipt.receiver_id {
                        try_refund_allowance(
                            state_update,
                            &receipt.receiver_id,
                            &action_receipt.signer_public_key,
                            transfer,
                        )?;
                    }
                } else {
                    // return Err(RuntimeError::InvalidTxError(InvalidTxError::SignerDoesNotExist{ signer_id: actor_id.clone()}));
                }
            }
            Action::AddKey(add_key) => {
                // metrics::ACTION_ADD_KEY_TOTAL.inc();
                action_add_key(
                    apply_state,
                    state_update,
                    account.as_mut().expect(EXPECT_ACCOUNT_EXISTS),
                    &mut result,
                    account_id,
                    add_key,
                )?;
            }
            Action::DeleteKey(delete_key) => {
                // metrics::ACTION_DELETE_KEY_TOTAL.inc();
                action_delete_key(
                    &apply_state.config.transaction_costs,
                    state_update,
                    account.as_mut().expect(EXPECT_ACCOUNT_EXISTS),
                    &mut result,
                    account_id,
                    delete_key,
                )?;
            }
            Action::FunctionCall(function_call) => {
                // metrics::ACTION_FUNCTION_CALL_TOTAL.inc();
                action_function_call(
                    state_update,
                    apply_state,
                    account.as_mut().expect(EXPECT_ACCOUNT_EXISTS),
                    receipt,
                    action_receipt,
                    promise_results,
                    &mut result,
                    account_id,
                    function_call,
                    action_hash,
                    &apply_state.config,
                    action_index + 1 == actions.len(),
                )?;
            }
            Action::DeleteAccount(delete_account) => {
                // metrics::ACTION_DELETE_ACCOUNT_TOTAL.inc();
                action_delete_account(
                    state_update,
                    account,
                    actor_id,
                    receipt,
                    &mut result,
                    account_id,
                    delete_account,
                )?;
            }
        };
        Ok(result)
    }

    // Executes when all Receipt `input_data_ids` are in the state
    fn apply_action_receipt(
        &self,
        state_update: &mut TrieUpdate,
        apply_state: &ApplyState,
        receipt: &Receipt,
        outgoing_receipts: &mut Vec<Receipt>,
        stats: &mut ApplyStats,
    ) -> Result<ExecutionOutcomeWithId, RuntimeError> {
        let action_receipt = match &receipt.receipt {
            ReceiptEnum::Action(action_receipt) => action_receipt,
            _ => unreachable!("given receipt should be an action receipt"),
        };

        let account_id = &receipt.receiver_id;
        // Collecting input data and removing it from the state
        let promise_results = action_receipt
            .input_data_ids
            .iter()
            .map(|data_id| {
                let ReceivedData { data } = get_received_data(state_update, account_id, *data_id)?
                    .ok_or_else(|| {
                        StorageError::StorageInconsistentState(
                            "received data should be in the state".to_string(),
                        )
                    })?;
                state_update.remove(TrieKey::ReceivedData {
                    receiver_id: account_id.clone(),
                    data_id: *data_id,
                });
                match data {
                    Some(value) => Ok(PromiseResult::Successful(value)),
                    None => Ok(PromiseResult::Failed),
                }
            })
            .collect::<Result<Vec<PromiseResult>, RuntimeError>>()?;

        // state_update might already have some updates so we need to make sure we commit it before
        // executing the actual receipt
        state_update.commit(StateChangeCause::ActionReceiptProcessingStarted {
            receipt_hash: receipt.get_hash(),
        });

        let mut account = get_account(state_update, account_id)?;
        let mut actor_id = receipt.predecessor_id.clone();
        let mut result = ActionResult::default();
        let exec_fee =
            apply_state.config.transaction_costs.action_receipt_creation_config.exec_fee();
        result.gas_used = exec_fee;
        result.gas_burnt = exec_fee;
        // Executing actions one by one
        for (action_index, action) in action_receipt.actions.iter().enumerate() {
            let action_hash = create_action_hash(
                receipt,
                action_index,
            );
            let mut new_result = self.apply_action(
                action,
                state_update,
                apply_state,
                &mut account,
                &mut actor_id,
                receipt,
                action_receipt,
                &promise_results,
                &action_hash,
                action_index,
                &action_receipt.actions,
            )?;

            if new_result.result.is_ok() {
                if let Err(e) = new_result.new_receipts.iter().try_for_each(|receipt| {
                    validate_receipt(&apply_state.config.wasm_config.limit_config, receipt)
                }) {
                    new_result.result = Err(ActionErrorKind::NewReceiptValidationError(e).into());
                }
            }
            result.merge(new_result)?;
            // TODO storage error
            if let Err(ref mut res) = result.result {
                res.index = Some(action_index as u64);
                break;
            }
        }

        // Going to check balance covers account's storage.
        if result.result.is_ok() {
            if let Some(ref mut account) = account {
                set_account(state_update, account_id.clone(), account);
            }
        }

        let gas_deficit_amount = if AccountId::is_system(&receipt.predecessor_id) {
            // We will set gas_burnt for refund receipts to be 0 when we calculate tx_burnt_amount
            // Here we don't set result.gas_burnt to be zero if CountRefundReceiptsInGasLimit is
            // enabled because we want it to be counted in gas limit calculation later
            
            // TODO: maybe we should count refund receipts in gas limit calculation
            // result.gas_burnt = 0;
            // result.gas_used = 0;

            // If the refund fails tokens are burned.
            if result.result.is_err() {
                stats.other_burnt_amount = safe_add_balance(
                    stats.other_burnt_amount,
                    total_deposit(&action_receipt.actions)?,
                )?
            }
            0
        } else {
            // Calculating and generating refunds
            self.generate_refund_receipts(
                apply_state.gas_price,
                receipt,
                action_receipt,
                &mut result,
                &apply_state.config.transaction_costs,
            )?
        };
        stats.gas_deficit_amount = safe_add_balance(stats.gas_deficit_amount, gas_deficit_amount)?;

        // Committing or rolling back state.
        match &result.result {
            Ok(_) => {
                state_update.commit(StateChangeCause::ReceiptProcessing {
                    receipt_hash: receipt.get_hash(),
                });
            }
            Err(_) => {
                state_update.rollback();
            }
        };

        // If the receipt is a refund, then we consider it free without burnt gas.
        let gas_burnt: Gas =
            if AccountId::is_system(&receipt.predecessor_id) { 0 } else { result.gas_burnt };
        // `gas_deficit_amount` is strictly less than `gas_price * gas_burnt`.
        let mut tx_burnt_amount =
            safe_gas_to_balance(apply_state.gas_price, gas_burnt)? - gas_deficit_amount;
        // The amount of tokens burnt for the execution of this receipt. It's used in the execution
        // outcome.
        let tokens_burnt = tx_burnt_amount;

        // Adding burnt gas reward for function calls if the account exists.
        let receiver_gas_reward = result.gas_burnt_for_function_call
            * *apply_state.config.transaction_costs.burnt_gas_reward.numer() as u64
            / *apply_state.config.transaction_costs.burnt_gas_reward.denom() as u64;
        // The balance that the current account should receive as a reward for function call
        // execution.
        let receiver_reward = safe_gas_to_balance(apply_state.gas_price, receiver_gas_reward)?
            .saturating_sub(gas_deficit_amount);
        if receiver_reward > 0 {
            let mut account = get_account(state_update, account_id)?;
            if let Some(ref mut account) = account {
                // Validators receive the remaining execution reward that was not given to the
                // account holder. If the account doesn't exist by the end of the execution, the
                // validators receive the full reward.
                tx_burnt_amount -= receiver_reward;
                account.set_amount(safe_add_balance(account.amount(), receiver_reward)?);
                set_account(state_update, account_id.clone(), account);
                state_update.commit(StateChangeCause::ActionReceiptGasReward {
                    receipt_hash: receipt.get_hash(),
                });
            }
        }

        stats.tx_burnt_amount = safe_add_balance(stats.tx_burnt_amount, tx_burnt_amount)?;

        // Generating outgoing data
        // A {
        // B().then(C())}  B--data receipt->C

        // A {
        // B(); 42}
        if !action_receipt.output_data_receivers.is_empty() {
            if let Ok(ReturnData::ReceiptIndex(receipt_index)) = result.result {
                // Modifying a new receipt instead of sending data
                match result
                    .new_receipts
                    .get_mut(receipt_index as usize)
                    .expect("the receipt for the given receipt index should exist")
                    .receipt
                {
                    ReceiptEnum::Action(ref mut new_action_receipt) => new_action_receipt
                        .output_data_receivers
                        .extend_from_slice(&action_receipt.output_data_receivers),
                    _ => unreachable!("the receipt should be an action receipt"),
                }
            } else {
                let data = match result.result {
                    Ok(ReturnData::Value(ref data)) => Some(data.clone()),
                    Ok(_) => Some(vec![]),
                    Err(_) => None,
                };
                result.new_receipts.extend(action_receipt.output_data_receivers.iter().map(
                    |data_receiver| Receipt {
                        predecessor_id: account_id.clone(),
                        receiver_id: data_receiver.receiver_id.clone(),
                        receipt_id: CryptoHash::default(),
                        receipt: ReceiptEnum::Data(DataReceipt {
                            data_id: data_receiver.data_id,
                            data: data.clone(),
                        }),
                    },
                ));
            };
        }

        // Generating receipt IDs
        let receipt_ids = result
            .new_receipts
            .into_iter()
            .enumerate()
            .filter_map(|(receipt_index, mut new_receipt)| {
                let receipt_id = create_receipt_id_from_receipt(
                    receipt,
                    receipt_index,
                );

                new_receipt.receipt_id = receipt_id;
                let is_action = matches!(&new_receipt.receipt, ReceiptEnum::Action(_));
                outgoing_receipts.push(new_receipt);
                if is_action {
                    Some(receipt_id)
                } else {
                    None
                }
            })
            .collect();

        let status = match result.result {
            Ok(ReturnData::ReceiptIndex(receipt_index)) => {
                ExecutionStatus::SuccessReceiptId(create_receipt_id_from_receipt(
                    receipt,
                    receipt_index as usize,
                ))
            }
            Ok(ReturnData::Value(data)) => ExecutionStatus::SuccessValue(data),
            Ok(ReturnData::None) => ExecutionStatus::SuccessValue(vec![]),
            Err(e) => ExecutionStatus::Failure(TxExecutionError::ActionError(e)),
        };

        Self::print_log(&result.logs);

        Ok(ExecutionOutcomeWithId {
            id: receipt.receipt_id,
            outcome: ExecutionOutcome {
                status,
                logs: result.logs,
                receipt_ids,
                gas_burnt: result.gas_burnt,
                tokens_burnt,
                executor_id: account_id.clone(),
                profile_data: Some(result.profile),
            },
        })
    }

    fn generate_refund_receipts(
        &self,
        current_gas_price: Balance,
        receipt: &Receipt,
        action_receipt: &ActionReceipt,
        result: &mut ActionResult,
        transaction_costs: &RuntimeFeesConfig,
    ) -> Result<Balance, RuntimeError> {
        let total_deposit = total_deposit(&action_receipt.actions)?;
        let prepaid_gas = total_prepaid_gas(&action_receipt.actions)?;
        let prepaid_exec_gas = safe_add_gas(
            total_prepaid_exec_fees(
                transaction_costs,
                &action_receipt.actions,
                &receipt.receiver_id,
            )?,
            transaction_costs.action_receipt_creation_config.exec_fee(),
        )?;
        let deposit_refund = if result.result.is_err() { total_deposit } else { 0 };
        let gas_refund = if result.result.is_err() {
            safe_add_gas(prepaid_gas, prepaid_exec_gas)? - result.gas_burnt
        } else {
            safe_add_gas(prepaid_gas, prepaid_exec_gas)? - result.gas_used
        };
        // Refund for the unused portion of the gas at the price at which this gas was purchased.
        let mut gas_balance_refund = safe_gas_to_balance(action_receipt.gas_price, gas_refund)?;
        let mut gas_deficit_amount = 0;
        if current_gas_price > action_receipt.gas_price {
            // In a rare scenario, when the current gas price is higher than the purchased gas
            // price, the difference is subtracted from the refund. If the refund doesn't have
            // enough balance to cover the difference, then the remaining balance is considered
            // the deficit and it's reported in the stats for the balance checker.
            gas_deficit_amount = safe_gas_to_balance(
                current_gas_price - action_receipt.gas_price,
                result.gas_burnt,
            )?;
            if gas_balance_refund >= gas_deficit_amount {
                gas_balance_refund -= gas_deficit_amount;
                gas_deficit_amount = 0;
            } else {
                gas_deficit_amount -= gas_balance_refund;
                gas_balance_refund = 0;
            }
        } else {
            // Refund for the difference of the purchased gas price and the the current gas price.
            gas_balance_refund = safe_add_balance(
                gas_balance_refund,
                safe_gas_to_balance(
                    action_receipt.gas_price - current_gas_price,
                    result.gas_burnt,
                )?,
            )?;
        }
        
        if deposit_refund > 0 {
            result
                .new_receipts
                .push(Receipt::new_balance_refund(&receipt.predecessor_id, deposit_refund));
        }
        
        if gas_balance_refund > 0 {
            // Gas refunds refund the allowance of the access key, so if the key exists on the
            // account it will increase the allowance by the refund amount.
            result.new_receipts.push(Receipt::new_gas_refund(
                &action_receipt.signer_id,
                gas_balance_refund,
                action_receipt.signer_public_key.clone(),
            ));
        }

        Ok(gas_deficit_amount)
    }

    fn process_receipt(
        &self,
        state_update: &mut TrieUpdate,
        apply_state: &ApplyState,
        receipt: &Receipt,
        outgoing_receipts: &mut Vec<Receipt>,
        stats: &mut ApplyStats,
    ) -> Result<Option<ExecutionOutcomeWithId>, RuntimeError> {
        let account_id = &receipt.receiver_id;

        match receipt.receipt {
            ReceiptEnum::Data(ref data_receipt) => {
                // Received a new data receipt.
                // Saving the data into the state keyed by the data_id.
                set_received_data(
                    state_update,
                    account_id.clone(),
                    data_receipt.data_id,
                    &ReceivedData { data: data_receipt.data.clone() },
                );
                // Check if there is already a receipt that was postponed and was awaiting for the
                // given data_id.
                // If we don't have a postponed receipt yet, we don't need to do anything for now.
                if let Some(receipt_id) = get(
                    state_update,
                    &TrieKey::PostponedReceiptId {
                        receiver_id: account_id.clone(),
                        data_id: data_receipt.data_id,
                    },
                )? {
                    // There is already a receipt that is awaiting for the just received data.
                    // Removing this pending data_id for the receipt from the state.
                    state_update.remove(TrieKey::PostponedReceiptId {
                        receiver_id: account_id.clone(),
                        data_id: data_receipt.data_id,
                    });
                    // Checking how many input data items is pending for the receipt.
                    let pending_data_count: u32 = get(
                        state_update,
                        &TrieKey::PendingDataCount { receiver_id: account_id.clone(), receipt_id },
                    )?
                    .ok_or_else(|| {
                        StorageError::StorageInconsistentState(
                            "pending data count should be in the state".to_string(),
                        )
                    })?;
                    if pending_data_count == 1 {
                        // It was the last input data pending for this receipt. We'll cleanup
                        // some receipt related fields from the state and execute the receipt.

                        // Removing pending data count from the state.
                        state_update.remove(TrieKey::PendingDataCount {
                            receiver_id: account_id.clone(),
                            receipt_id,
                        });
                        // Fetching the receipt itself.
                        let ready_receipt =
                            get_postponed_receipt(state_update, account_id, receipt_id)?
                                .ok_or_else(|| {
                                    StorageError::StorageInconsistentState(
                                        "pending receipt should be in the state".to_string(),
                                    )
                                })?;
                        // Removing the receipt from the state.
                        remove_postponed_receipt(state_update, account_id, receipt_id);
                        // Executing the receipt. It will read all the input data and clean it up
                        // from the state.
                        return self
                            .apply_action_receipt(
                                state_update,
                                apply_state,
                                &ready_receipt,
                                outgoing_receipts,
                                stats,
                            )
                            .map(Some);
                    } else {
                        // There is still some pending data for the receipt, so we update the
                        // pending data count in the state.
                        set(
                            state_update,
                            TrieKey::PendingDataCount {
                                receiver_id: account_id.clone(),
                                receipt_id,
                            },
                            &(pending_data_count.checked_sub(1).ok_or_else(|| {
                                StorageError::StorageInconsistentState(
                                    "pending data count is 0, but there is a new DataReceipt"
                                        .to_string(),
                                )
                            })?),
                        );
                    }
                }
            }
            ReceiptEnum::Action(ref action_receipt) => {
                // Received a new action receipt. We'll first check how many input data items
                // were already received before and saved in the state.
                // And if we have all input data, then we can immediately execute the receipt.
                // If not, then we will postpone this receipt for later.
                let mut pending_data_count: u32 = 0;
                for data_id in &action_receipt.input_data_ids {
                    if get_received_data(state_update, account_id, *data_id)?.is_none() {
                        pending_data_count += 1;
                        // The data for a given data_id is not available, so we save a link to this
                        // receipt_id for the pending data_id into the state.
                        set(
                            state_update,
                            TrieKey::PostponedReceiptId {
                                receiver_id: account_id.clone(),
                                data_id: *data_id,
                            },
                            &receipt.receipt_id,
                        )
                    }
                }
                if pending_data_count == 0 {
                    // All input data is available. Executing the receipt. It will cleanup
                    // input data from the state.
                    return self
                        .apply_action_receipt(
                            state_update,
                            apply_state,
                            receipt,
                            outgoing_receipts,
                            stats,
                        )
                        .map(Some);
                } else {
                    // Not all input data is available now.
                    // Save the counter for the number of pending input data items into the state.
                    set(
                        state_update,
                        TrieKey::PendingDataCount {
                            receiver_id: account_id.clone(),
                            receipt_id: receipt.receipt_id,
                        },
                        &pending_data_count,
                    );
                    // Save the receipt itself into the state.
                    set_postponed_receipt(state_update, receipt);
                }
            }
        };
        // We didn't trigger execution, so we need to commit the state.
        state_update
            .commit(StateChangeCause::PostponedReceipt { receipt_hash: receipt.get_hash() });
        Ok(None)
    }

    /// Applies new singed transactions and incoming receipts for some chunk/shard on top of
    /// given trie and the given state root.
    /// If the validator accounts update is provided, updates validators accounts.
    /// All new signed transactions should be valid and already verified by the chunk producer.
    /// If any transaction is invalid, it would return an `InvalidTxError`.
    /// Returns an `ApplyResult` that contains the new state root, trie changes,
    /// new outgoing receipts, execution outcomes for
    /// all transactions, local action receipts (generated from transactions with signer ==
    /// receivers) and incoming action receipts.
    pub fn apply(
        &self,
        trie: Trie,
        root: CryptoHash,
        apply_state: &ApplyState,
        incoming_receipts: &[Receipt],
        transactions: &[SignedTransaction],
    ) -> Result<ApplyResult, RuntimeError> {
        let _span = tracing::debug_span!(target: "runtime", "Runtime::apply").entered();

        let trie = Rc::new(trie);

        // TODO:: maybe we can use StoreUpdate instead of TrieUpdate
        let initial_state = TrieUpdate::new(trie.clone(), root);
        let mut state_update = TrieUpdate::new(trie.clone(), root);

        let mut stats = ApplyStats::default();

        let mut outgoing_receipts = Vec::new();
        let mut local_receipts = vec![];
        let mut outcomes = vec![];
        let mut processed_delayed_receipts = vec![];
        // This contains the gas "burnt" for refund receipts. Even though we don't actually
        // charge any gas for refund receipts, we still count the gas use towards the block gas
        // limit
        // TODO: gas_used_for_migrations
        let mut total_gas_burnt = 0;

        for signed_transaction in transactions {
            let (receipt, outcome_with_id) = self.process_transaction(
                &mut state_update,
                apply_state,
                signed_transaction,
                &mut stats,
            )?;
            if receipt.receiver_id == signed_transaction.transaction.signer_id {
                local_receipts.push(receipt);
            } else {
                outgoing_receipts.push(receipt);
            }

            total_gas_burnt += outcome_with_id.outcome.gas_burnt;

            outcomes.push(outcome_with_id);
        }

        let mut delayed_receipts_indices: DelayedReceiptIndices =
            get(&state_update, &TrieKey::DelayedReceiptIndices)?.unwrap_or_default();
        let initial_delayed_receipt_indices = delayed_receipts_indices.clone();

        let mut process_receipt = |receipt: &Receipt,
                                   state_update: &mut TrieUpdate,
                                   total_gas_burnt: &mut Gas|
         -> Result<_, RuntimeError> {
            // let _span = tracing::debug_span!(target: "runtime", "Runtime::process_receipt", receipt_id = %receipt.receipt_id, node_counter = state_update.trie.counter.get()).entered();
            let result = self.process_receipt(
                state_update,
                apply_state,
                receipt,
                &mut outgoing_receipts,
                &mut stats,
            );
            tracing::debug!(target: "runtime", node_counter = state_update.trie.counter.get());
            result?.into_iter().try_for_each(
                |outcome_with_id: ExecutionOutcomeWithId| -> Result<(), RuntimeError> {
                    *total_gas_burnt =
                        safe_add_gas(*total_gas_burnt, outcome_with_id.outcome.gas_burnt)?;
                    outcomes.push(outcome_with_id);
                    Ok(())
                },
            )?;
            Ok(())
        };

        let gas_limit = apply_state.gas_limit.unwrap_or(Gas::max_value());

        // We first process local receipts. They contain staking, local contract calls, etc.
        for receipt in local_receipts.iter() {
            if total_gas_burnt < gas_limit {
                // NOTE: We don't need to validate the local receipt, because it's just validated in
                // the `verify_and_charge_transaction`.
                process_receipt(receipt, &mut state_update, &mut total_gas_burnt)?;
            } else {
                Self::delay_receipt(&mut state_update, &mut delayed_receipts_indices, receipt)?;
            }
        }

        // Then we process the delayed receipts. It's a backlog of receipts from the past blocks.
        while delayed_receipts_indices.first_index < delayed_receipts_indices.next_available_index {
            if total_gas_burnt >= gas_limit {
                break;
            }
            let key = TrieKey::DelayedReceipt { index: delayed_receipts_indices.first_index };
            let receipt: Receipt = get(&state_update, &key)?.ok_or_else(|| {
                StorageError::StorageInconsistentState(format!(
                    "Delayed receipt #{} should be in the state",
                    delayed_receipts_indices.first_index
                ))
            })?;

            // Validating the delayed receipt. If it fails, it's likely the state is inconsistent.
            validate_receipt(&apply_state.config.wasm_config.limit_config, &receipt).map_err(
                |e| {
                    StorageError::StorageInconsistentState(format!(
                        "Delayed receipt #{} in the state is invalid: {}",
                        delayed_receipts_indices.first_index, e
                    ))
                },
            )?;

            state_update.remove(key);
            // Math checked above: first_index is less than next_available_index
            delayed_receipts_indices.first_index += 1;
            process_receipt(&receipt, &mut state_update, &mut total_gas_burnt)?;
            processed_delayed_receipts.push(receipt);
        }

        // And then we process the new incoming receipts. These are receipts from other shards.
        for receipt in incoming_receipts.iter() {
            // Validating new incoming no matter whether we have available gas or not. We don't
            // want to store invalid receipts in state as delayed.
            validate_receipt(&apply_state.config.wasm_config.limit_config, receipt)
                .map_err(RuntimeError::ReceiptValidationError)?;
            
            if total_gas_burnt < gas_limit {
                process_receipt(receipt, &mut state_update, &mut total_gas_burnt)?;
            } else {
                Self::delay_receipt(&mut state_update, &mut delayed_receipts_indices, receipt)?;
            }
        }

        if delayed_receipts_indices != initial_delayed_receipt_indices {
            set(&mut state_update, TrieKey::DelayedReceiptIndices, &delayed_receipts_indices);
        }

        check_balance(
            &apply_state.config.transaction_costs,
            &initial_state,
            &state_update,
            incoming_receipts,
            transactions,
            &outgoing_receipts,
            &stats,
        )?;

        state_update.commit(StateChangeCause::UpdatedDelayedReceipts);

        let (trie_changes, state_changes) = state_update.finalize()?;

        let state_root = trie_changes.new_root;
        let proof = trie.recorded_storage();
        Ok(ApplyResult {
            state_root,
            trie_changes,
            outgoing_receipts,
            outcomes,
            state_changes,
            stats,
            processed_delayed_receipts,
            proof,
        })
    }

    // Adds the given receipt into the end of the delayed receipt queue in the state.
    pub fn delay_receipt(
        state_update: &mut TrieUpdate,
        delayed_receipts_indices: &mut DelayedReceiptIndices,
        receipt: &Receipt,
    ) -> Result<(), StorageError> {
        set(
            state_update,
            TrieKey::DelayedReceipt { index: delayed_receipts_indices.next_available_index },
            receipt,
        );
        delayed_receipts_indices.next_available_index =
            delayed_receipts_indices.next_available_index.checked_add(1).ok_or_else(|| {
                StorageError::StorageInconsistentState(
                    "Next available index for delayed receipt exceeded the integer limit"
                        .to_string(),
                )
            })?;
        Ok(())
    }

    /// It's okay to use unsafe math here, because this method should only be called on the trusted
    /// state records (e.g. at launch from genesis)
    pub fn compute_storage_usage(
        &self,
        records: &[StateRecord],
        config: &RuntimeConfig,
    ) -> HashMap<AccountId, u64> {
        let mut result = HashMap::new();
        let config = &config.transaction_costs.storage_usage_config;
        for record in records {
            let account_and_storage = match record {
                StateRecord::Account { account_id, .. } => {
                    Some((account_id.clone(), config.num_bytes_account))
                }
                StateRecord::Data { account_id, data_key, value } => {
                    let storage_usage =
                        config.num_extra_bytes_record + data_key.len() as u64 + value.len() as u64;
                    Some((account_id.clone(), storage_usage))
                }
                StateRecord::Contract { account_id, code } => {
                    Some((account_id.clone(), code.len() as u64))
                }
                StateRecord::AccessKey { account_id, public_key, access_key } => {
                    let public_key: PublicKey = public_key.clone();
                    let access_key: AccessKey = access_key.clone().into();
                    let storage_usage = config.num_extra_bytes_record
                        + public_key.try_to_vec().unwrap().len() as u64
                        + access_key.try_to_vec().unwrap().len() as u64;
                    Some((account_id.clone(), storage_usage))
                }
                StateRecord::PostponedReceipt(_) => None,
                StateRecord::ReceivedData { .. } => None,
                StateRecord::DelayedReceipt(_) => None,
            };
            if let Some((account, storage_usage)) = account_and_storage {
                *result.entry(account).or_default() += storage_usage;
            }
        }
        result
    }

    /// Balances are account, publickey, initial_balance, initial_tx_stake
    pub fn apply_genesis_state (
        &self,
        tries: ShardTries,
        records: &[StateRecord],
        config: &RuntimeConfig,
    ) -> (StoreUpdate, StateRoot) {
        let mut state_update = tries.new_trie_update(CryptoHash::default());
        let mut postponed_receipts: Vec<Receipt> = vec![];
        let mut delayed_receipts_indices = DelayedReceiptIndices::default();
        for record in records {
            match record.clone() {
                StateRecord::Account { account_id, account } => {
                    set_account(&mut state_update, account_id, &account);
                }
                StateRecord::Data { account_id, data_key, value } => {
                    state_update.set(TrieKey::ContractData { key: data_key, account_id }, value);
                }
                StateRecord::Contract { account_id, code } => {
                    let code = ContractCode::new(&code);
                    set_code(&mut state_update, account_id, &code);
                }
                StateRecord::AccessKey { account_id, public_key, access_key } => {
                    set_access_key(&mut state_update, account_id, public_key, &access_key);
                }
                StateRecord::PostponedReceipt(receipt) => {
                    // Delaying processing postponed receipts, until we process all data first
                    postponed_receipts.push(*receipt);
                }
                StateRecord::ReceivedData { account_id, data_id, data } => {
                    set_received_data(
                        &mut state_update,
                        account_id,
                        data_id,
                        &ReceivedData { data },
                    );
                }
                StateRecord::DelayedReceipt(receipt) => {
                    Self::delay_receipt(
                        &mut state_update,
                        &mut delayed_receipts_indices,
                        &*receipt,
                    )
                    .unwrap();
                }
            }
        }
        for (account_id, storage_usage) in self.compute_storage_usage(records, &config) {
            let mut account = get_account(&state_update, &account_id)
                .expect("Genesis storage error")
                .expect("Account must exist");
            account.set_storage_usage(storage_usage);
            set_account(&mut state_update, account_id, &account);
        }
        // Processing postponed receipts after we stored all received data
        for receipt in postponed_receipts {
            let account_id = &receipt.receiver_id;
            let action_receipt = match &receipt.receipt {
                ReceiptEnum::Action(a) => a,
                _ => panic!("Expected action receipt"),
            };
            // Logic similar to `apply_receipt`
            let mut pending_data_count: u32 = 0;
            for data_id in &action_receipt.input_data_ids {
                if get_received_data(&state_update, account_id, *data_id)
                    .expect("Genesis storage error")
                    .is_none()
                {
                    pending_data_count += 1;
                    set(
                        &mut state_update,
                        TrieKey::PostponedReceiptId {
                            receiver_id: account_id.clone(),
                            data_id: *data_id,
                        },
                        &receipt.receipt_id,
                    )
                }
            }
            if pending_data_count == 0 {
                panic!("Postponed receipt should have pending data")
            } else {
                set(
                    &mut state_update,
                    TrieKey::PendingDataCount {
                        receiver_id: account_id.clone(),
                        receipt_id: receipt.receipt_id,
                    },
                    &pending_data_count,
                );
                set_postponed_receipt(&mut state_update, &receipt);
            }
        }
        if delayed_receipts_indices != DelayedReceiptIndices::default() {
            set(&mut state_update, TrieKey::DelayedReceiptIndices, &delayed_receipts_indices);
        }

        state_update.commit(StateChangeCause::InitialState);
        let trie_changes = state_update.finalize_genesis().expect("Genesis state update failed");

        let (store_update, state_root) = tries.apply_genesis(trie_changes);
        (store_update, state_root)
    }
}

#[cfg(test)]
mod tests {
    use skw_vm_store::set_access_key;
    use skw_vm_primitives::crypto::{InMemorySigner, KeyType};
    use skw_vm_primitives::crypto::PublicKey;
    use skw_vm_primitives::account::AccessKey;
    use skw_vm_primitives::contract_runtime::{
        hash_bytes, MerkleHash
    };
    use skw_vm_primitives::crypto::Signer;
    use skw_vm_primitives::test_utils::{account_new};
    use skw_vm_primitives::transaction::{
        FunctionCallAction, TransferAction, DeleteKeyAction, AddKeyAction,
    };
    use std::sync::Arc;
    use skw_vm_store::test_utils::create_tries;
    // use skw_vm_engine::get_contract_cache_key;

    pub fn alice_account() -> AccountId {
        "alice.near".parse().unwrap()
    }
    pub fn bob_account() -> AccountId {
        "bob.near".parse().unwrap()
    }

    use super::*;

    const GAS_PRICE: Balance = 5000;

    fn to_yocto(near: Balance) -> Balance {
        near * 10u128.pow(24)
    }

    fn create_receipts_with_actions(
        account_id: AccountId,
        signer: Arc<InMemorySigner>,
        actions: Vec<Action>,
    ) -> Vec<Receipt> {
        vec![Receipt {
            predecessor_id: account_id.clone(),
            receiver_id: account_id.clone(),
            receipt_id: CryptoHash::default(),
            receipt: ReceiptEnum::Action(ActionReceipt {
                signer_id: account_id,
                signer_public_key: signer.public_key(),
                gas_price: GAS_PRICE,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions,
            }),
        }]
    }


    #[test]
    fn test_get_and_set_accounts() {
        let tries = create_tries();
        let mut state_update =
            tries.new_trie_update(MerkleHash::default());
        let test_account = account_new(to_yocto(10), hash_bytes(&[]));
        let account_id = bob_account();
        set_account(&mut state_update, account_id.clone(), &test_account);
        let get_res = get_account(&state_update, &account_id).unwrap().unwrap();
        assert_eq!(test_account, get_res);
    }

    #[test]
    fn test_get_account_from_trie() {
        let tries = create_tries();
        let root = MerkleHash::default();
        let mut state_update = tries.new_trie_update(root);
        let test_account = account_new(to_yocto(10), hash_bytes(&[]));
        let account_id = bob_account();
        set_account(&mut state_update, account_id.clone(), &test_account);
        state_update.commit(StateChangeCause::InitialState);
        let trie_changes = state_update.finalize().unwrap().0;
        let (store_update, new_root) =
            tries.apply_all(&trie_changes).unwrap();
        store_update.commit().unwrap();
        let new_state_update = tries.new_trie_update(new_root);
        let get_res = get_account(&new_state_update, &account_id).unwrap().unwrap();
        assert_eq!(test_account, get_res);
    }

    /***************/
    /* Apply tests */
    /***************/

    fn setup_runtime(
        initial_balance: Balance,
        initial_locked: Balance,
        gas_limit: Gas,
    ) -> (Runtime, ShardTries, CryptoHash, ApplyState, Arc<InMemorySigner>)
    {
        let tries = create_tries();
        let root = MerkleHash::default();
        let runtime = Runtime::new();
        let account_id = alice_account();
        let signer = Arc::new(InMemorySigner::from_seed(
            account_id.clone(),
            KeyType::ED25519,
            account_id.as_ref(),
        ));

        let mut initial_state = tries.new_trie_update(root);
        let mut initial_account = account_new(initial_balance, hash_bytes(&[]));
        // For the account and a full access key
        initial_account.set_storage_usage(182);
        initial_account.set_locked(initial_locked);
        set_account(&mut initial_state, account_id.clone(), &initial_account);
        set_access_key(
            &mut initial_state,
            account_id,
            signer.public_key(),
            &AccessKey::full_access(),
        );
        initial_state.commit(StateChangeCause::InitialState);
        let trie_changes = initial_state.finalize().unwrap().0;
        let (store_update, root) =
            tries.apply_all(&trie_changes).unwrap();
        store_update.commit().unwrap();

        let apply_state = ApplyState {
            block_number: 1,
            prev_block_hash: Default::default(),
            block_hash: Default::default(),
            gas_price: GAS_PRICE,
            block_timestamp: 100,
            gas_limit: Some(gas_limit),
            random_seed: Default::default(),
            config: Arc::new(RuntimeConfig::test()),
        };

        (runtime, tries, root, apply_state, signer)
    }

    #[test]
    fn test_apply_no_op() {
        let (runtime, tries, root, apply_state, _) =
            setup_runtime(to_yocto(1_000_000), 0, 10u64.pow(15));
        runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &[],
                &[],
            )
            .unwrap();
    }

    #[test]
    fn test_apply_refund_receipts() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let small_transfer = to_yocto(10_000);
        let gas_limit = 1;
        let (runtime, tries, mut root, apply_state, _) =
            setup_runtime(initial_balance, initial_locked, gas_limit);

        let n = 10;
        let receipts = generate_refund_receipts(small_transfer, n);

        // Checking n receipts delayed
        for i in 1..=n + 3 {
            let prev_receipts: &[Receipt] = if i == 1 { &receipts } else { &[] };
            let apply_result = runtime
                .apply(
                    tries.get_trie(),
                    root,
                    &apply_state,
                    prev_receipts,
                    &[],
                )
                .unwrap();
            let (store_update, new_root) =
                tries.apply_all(&apply_result.trie_changes).unwrap();
            root = new_root;
            store_update.commit().unwrap();
            let state = tries.new_trie_update(root);
            let account = get_account(&state, &alice_account()).unwrap().unwrap();
            let capped_i = std::cmp::min(i, n);

            assert_eq!(
                account.amount(),
                initial_balance
                    + small_transfer * Balance::from(capped_i)
                    + Balance::from(capped_i * (capped_i - 1) / 2)
            );
        }
    }

    fn generate_refund_receipts(small_transfer: u128, n: u64) -> Vec<Receipt> {
        let mut receipt_id = CryptoHash::default();
        (0..n)
            .map(|i| {
                receipt_id = hash_bytes(receipt_id.as_ref());
                Receipt::new_balance_refund(&alice_account(), small_transfer + Balance::from(i))
            })
            .collect()
    }

    #[test]
    fn test_apply_delayed_receipts_feed_all_at_once() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let small_transfer = to_yocto(10_000);
        let gas_limit = 1;
        let (runtime, tries, mut root, apply_state, _) =
            setup_runtime(initial_balance, initial_locked, gas_limit);

        let n = 10;
        let receipts = generate_receipts(small_transfer, n);

        // Checking n receipts delayed by 1 + 3 extra
        for i in 1..=n + 3 {
            let prev_receipts: &[Receipt] = if i == 1 { &receipts } else { &[] };
            let apply_result = runtime
                .apply(
                    tries.get_trie(),
                    root,
                    &apply_state,
                    prev_receipts,
                    &[],
                )
                .unwrap();
            let (store_update, new_root) =
                tries.apply_all(&apply_result.trie_changes, ).unwrap();
            root = new_root;
            store_update.commit().unwrap();
            let state = tries.new_trie_update(root);
            let account = get_account(&state, &alice_account()).unwrap().unwrap();
            let capped_i = std::cmp::min(i, n);
            assert_eq!(
                account.amount(),
                initial_balance
                    + small_transfer * Balance::from(capped_i)
                    + Balance::from(capped_i * (capped_i - 1) / 2)
            );
        }
    }

    #[test]
    fn test_apply_delayed_receipts_add_more_using_chunks() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let small_transfer = to_yocto(10_000);
        let (runtime, tries, mut root, mut apply_state, _) =
            setup_runtime(initial_balance, initial_locked, 1);

        let receipt_gas_cost = apply_state
            .config
            .transaction_costs
            .action_receipt_creation_config
            .exec_fee()
            + apply_state.config.transaction_costs.action_creation_config.transfer_cost.exec_fee();
        apply_state.gas_limit = Some(receipt_gas_cost * 3);

        let n = 40;
        let receipts = generate_receipts(small_transfer, n);
        let mut receipt_chunks = receipts.chunks_exact(4);

        // Every time we'll process 3 receipts, so we need n / 3 rounded up. Then we do 3 extra.
        for i in 1..=n / 3 + 3 {
            let prev_receipts: &[Receipt] = receipt_chunks.next().unwrap_or_default();
            let apply_result = runtime
                .apply(
                    tries.get_trie(),
                    root,
                    &apply_state,
                    prev_receipts,
                    &[],
                )
                .unwrap();
            let (store_update, new_root) =
                tries.apply_all(&apply_result.trie_changes, ).unwrap();
            root = new_root;
            store_update.commit().unwrap();
            let state = tries.new_trie_update(root);
            let account = get_account(&state, &alice_account()).unwrap().unwrap();
            let capped_i = std::cmp::min(i * 3, n);
            assert_eq!(
                account.amount(),
                initial_balance
                    + small_transfer * Balance::from(capped_i)
                    + Balance::from(capped_i * (capped_i - 1) / 2)
            );
        }
    }

    #[test]
    fn test_apply_delayed_receipts_adjustable_gas_limit() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let small_transfer = to_yocto(10_000);
        let (runtime, tries, mut root, mut apply_state, _) =
            setup_runtime(initial_balance, initial_locked, 1);

        let receipt_gas_cost = apply_state
            .config
            .transaction_costs
            .action_receipt_creation_config
            .exec_fee()
            + apply_state.config.transaction_costs.action_creation_config.transfer_cost.exec_fee();

        let n = 120;
        let receipts = generate_receipts(small_transfer, n);
        let mut receipt_chunks = receipts.chunks_exact(4);

        let mut num_receipts_given = 0;
        let mut num_receipts_processed = 0;
        let mut num_receipts_per_block = 1;
        // Test adjusts gas limit based on the number of receipt given and number of receipts processed.
        while num_receipts_processed < n {
            if num_receipts_given > num_receipts_processed {
                num_receipts_per_block += 1;
            } else if num_receipts_per_block > 1 {
                num_receipts_per_block -= 1;
            }
            apply_state.gas_limit = Some(num_receipts_per_block * receipt_gas_cost);
            let prev_receipts: &[Receipt] = receipt_chunks.next().unwrap_or_default();
            num_receipts_given += prev_receipts.len() as u64;
            let apply_result = runtime
                .apply(
                    tries.get_trie(),
                    root,
                    &apply_state,
                    prev_receipts,
                    &[],
                )
                .unwrap();
            let (store_update, new_root) =
                tries.apply_all(&apply_result.trie_changes, ).unwrap();
            root = new_root;
            store_update.commit().unwrap();
            let state = tries.new_trie_update(root);
            num_receipts_processed += apply_result.outcomes.len() as u64;
            let account = get_account(&state, &alice_account()).unwrap().unwrap();
            assert_eq!(
                account.amount(),
                initial_balance
                    + small_transfer * Balance::from(num_receipts_processed)
                    + Balance::from(num_receipts_processed * (num_receipts_processed - 1) / 2)
            );
            println!(
                "{} processed out of {} given. With limit {} receipts per block",
                num_receipts_processed, num_receipts_given, num_receipts_per_block
            );
        }
    }

    fn generate_receipts(small_transfer: u128, n: u64) -> Vec<Receipt> {
        let mut receipt_id = CryptoHash::default();
        (0..n)
            .map(|i| {
                receipt_id = hash_bytes(receipt_id.as_ref());
                Receipt {
                    predecessor_id: bob_account(),
                    receiver_id: alice_account(),
                    receipt_id,
                    receipt: ReceiptEnum::Action(ActionReceipt {
                        signer_id: bob_account(),
                        signer_public_key: PublicKey::empty(KeyType::ED25519),
                        gas_price: GAS_PRICE,
                        output_data_receivers: vec![],
                        input_data_ids: vec![],
                        actions: vec![Action::Transfer(TransferAction {
                            deposit: small_transfer + Balance::from(i),
                        })],
                    }),
                }
            })
            .collect()
    }
    
    #[test]
    fn test_apply_delayed_receipts_local_tx() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let small_transfer = to_yocto(10_000);
        let (runtime, tries, root, mut apply_state, signer) =
            setup_runtime(initial_balance, initial_locked, 1);

        let receipt_exec_gas_fee = 1000;
        let mut free_config = RuntimeConfig::free();
        free_config.transaction_costs.action_receipt_creation_config.execution =
            receipt_exec_gas_fee;
        apply_state.config = Arc::new(free_config);
        // This allows us to execute 3 receipts per apply.
        apply_state.gas_limit = Some(receipt_exec_gas_fee * 3);

        let num_receipts = 6;
        let receipts = generate_receipts(small_transfer, num_receipts);

        let num_transactions = 9;
        let local_transactions = (0..num_transactions)
            .map(|i| {
                SignedTransaction::send_money(
                    i + 1,
                    alice_account(),
                    alice_account(),
                    &*signer,
                    small_transfer,
                    CryptoHash::default(),
                )
            })
            .collect::<Vec<_>>();

        // STEP #1. Pass 4 new local transactions + 2 receipts.
        // We can process only 3 local TX receipts TX#0, TX#1, TX#2.
        // TX#3 receipt and R#0, R#1 are delayed.
        // The new delayed queue is TX#3, R#0, R#1.
        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts[0..2],
                &local_transactions[0..4],
            )
            .unwrap();
        let (store_update, root) =
            tries.apply_all(&apply_result.trie_changes, ).unwrap();
        store_update.commit().unwrap();

        assert_eq!(
            apply_result.outcomes.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![
                local_transactions[0].get_hash(), // tx 0
                local_transactions[1].get_hash(), // tx 1
                local_transactions[2].get_hash(), // tx 2
                local_transactions[3].get_hash(), // tx 3 - the TX is processed, but the receipt is delayed
                create_receipt_id_from_transaction(
                    &local_transactions[0],
                ), // receipt for tx 0
                create_receipt_id_from_transaction(
                    &local_transactions[1],
                ), // receipt for tx 1
                create_receipt_id_from_transaction(
                    &local_transactions[2],
                ), // receipt for tx 2
            ],
            "STEP #1 failed",
        );

        // STEP #2. Pass 1 new local transaction (TX#4) + 1 receipts R#2.
        // We process 1 local receipts for TX#4, then delayed TX#3 receipt and then receipt R#0.
        // R#2 is added to delayed queue.
        // The new delayed queue is R#1, R#2
        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts[2..3],
                &local_transactions[4..5],
            )
            .unwrap();
        let (store_update, root) =
            tries.apply_all(&apply_result.trie_changes, ).unwrap();
        store_update.commit().unwrap();

        assert_eq!(
            apply_result.outcomes.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![
                local_transactions[4].get_hash(), // tx 4
                create_receipt_id_from_transaction(
                    &local_transactions[4],
                ), // receipt for tx 4
                create_receipt_id_from_transaction(
                    &local_transactions[3],
                ), // receipt for tx 3
                receipts[0].receipt_id,           // receipt #0
            ],
            "STEP #2 failed",
        );

        // STEP #3. Pass 4 new local transaction (TX#5, TX#6, TX#7, TX#8) and 1 new receipt R#3.
        // We process 3 local receipts for TX#5, TX#6, TX#7.
        // TX#8 and R#3 are added to delayed queue.
        // The new delayed queue is R#1, R#2, TX#8, R#3
        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts[3..4],
                &local_transactions[5..9],
            )
            .unwrap();
        let (store_update, root) =
            tries.apply_all(&apply_result.trie_changes, ).unwrap();
        store_update.commit().unwrap();

        assert_eq!(
            apply_result.outcomes.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![
                local_transactions[5].get_hash(), // tx 5
                local_transactions[6].get_hash(), // tx 6
                local_transactions[7].get_hash(), // tx 7
                local_transactions[8].get_hash(), // tx 8
                create_receipt_id_from_transaction(
                    &local_transactions[5],
                ), // receipt for tx 5
                create_receipt_id_from_transaction(
                    &local_transactions[6],
                ), // receipt for tx 6
                create_receipt_id_from_transaction(
                    &local_transactions[7],
                ), // receipt for tx 7
            ],
            "STEP #3 failed",
        );

        // STEP #4. Pass no new TXs and 1 receipt R#4.
        // We process R#1, R#2, TX#8.
        // R#4 is added to delayed queue.
        // The new delayed queue is R#3, R#4
        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts[4..5],
                &[],
            )
            .unwrap();
        let (store_update, root) =
            tries.apply_all(&apply_result.trie_changes, ).unwrap();
        store_update.commit().unwrap();

        assert_eq!(
            apply_result.outcomes.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![
                receipts[1].receipt_id, // receipt #1
                receipts[2].receipt_id, // receipt #2
                create_receipt_id_from_transaction(
                    &local_transactions[8],
                ), // receipt for tx 8
            ],
            "STEP #4 failed",
        );

        // STEP #5. Pass no new TXs and 1 receipt R#5.
        // We process R#3, R#4, R#5.
        // The new delayed queue is empty.
        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts[5..6],
                &[],
            )
            .unwrap();

        assert_eq!(
            apply_result.outcomes.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![
                receipts[3].receipt_id, // receipt #3
                receipts[4].receipt_id, // receipt #4
                receipts[5].receipt_id, // receipt #5
            ],
            "STEP #5 failed",
        );
    }

    #[test]
    fn test_apply_deficit_gas_for_transfer() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let small_transfer = to_yocto(10_000);
        let gas_limit = 10u64.pow(15);
        let (runtime, tries, root, apply_state, _) =
            setup_runtime(initial_balance, initial_locked, gas_limit);

        let n = 1;
        let mut receipts = generate_receipts(small_transfer, n);
        if let ReceiptEnum::Action(action_receipt) = &mut receipts.get_mut(0).unwrap().receipt {
            action_receipt.gas_price = GAS_PRICE / 10;
        }

        let result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts,
                &[],
            )
            .unwrap();
        assert_eq!(result.stats.gas_deficit_amount, result.stats.tx_burnt_amount * 9)
    }

    #[test]
    fn test_apply_deficit_gas_for_function_call_covered() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let gas_limit = 10u64.pow(15);
        let (runtime, tries, root, apply_state, _) =
            setup_runtime(initial_balance, initial_locked, gas_limit);

        let gas = 2 * 10u64.pow(14);
        let gas_price = GAS_PRICE / 10;
        let actions = vec![Action::FunctionCall(FunctionCallAction {
            method_name: "hello".to_string(),
            args: b"world".to_vec(),
            gas,
            deposit: 0,
        })];

        let expected_gas_burnt = safe_add_gas(
            apply_state.config.transaction_costs.action_receipt_creation_config.exec_fee(),
            total_prepaid_exec_fees(
                &apply_state.config.transaction_costs,
                &actions,
                &alice_account(),
            )
            .unwrap(),
        )
        .unwrap();
        let receipts = vec![Receipt {
            predecessor_id: bob_account(),
            receiver_id: alice_account(),
            receipt_id: CryptoHash::default(),
            receipt: ReceiptEnum::Action(ActionReceipt {
                signer_id: bob_account(),
                signer_public_key: PublicKey::empty(KeyType::ED25519),
                gas_price,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions,
            }),
        }];
        let total_receipt_cost = Balance::from(gas + expected_gas_burnt) * gas_price;
        let expected_gas_burnt_amount = Balance::from(expected_gas_burnt) * GAS_PRICE;
        let expected_refund = total_receipt_cost - expected_gas_burnt_amount;

        let result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts,
                &[],
            )
            .unwrap();
        // We used part of the prepaid gas to paying extra fees.
        assert_eq!(result.stats.gas_deficit_amount, 0);
        // The refund is less than the received amount.
        match &result.outgoing_receipts[0].receipt {
            ReceiptEnum::Action(ActionReceipt { actions, .. }) => {
                assert!(
                    matches!(actions[0], Action::Transfer(TransferAction { deposit }) if deposit == expected_refund)
                );
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_apply_deficit_gas_for_function_call_partial() {
        let initial_balance = to_yocto(1_000_000);
        let initial_locked = to_yocto(500_000);
        let gas_limit = 10u64.pow(15);
        let (runtime, tries, root, apply_state, _) =
            setup_runtime(initial_balance, initial_locked, gas_limit);

        let gas = 1_000_000;
        let gas_price = GAS_PRICE / 10;
        let actions = vec![Action::FunctionCall(FunctionCallAction {
            method_name: "hello".to_string(),
            args: b"world".to_vec(),
            gas,
            deposit: 0,
        })];

        let expected_gas_burnt = safe_add_gas(
            apply_state.config.transaction_costs.action_receipt_creation_config.exec_fee(),
            total_prepaid_exec_fees(
                &apply_state.config.transaction_costs,
                &actions,
                &alice_account(),
            )
            .unwrap(),
        )
        .unwrap();
        let receipts = vec![Receipt {
            predecessor_id: bob_account(),
            receiver_id: alice_account(),
            receipt_id: CryptoHash::default(),
            receipt: ReceiptEnum::Action(ActionReceipt {
                signer_id: bob_account(),
                signer_public_key: PublicKey::empty(KeyType::ED25519),
                gas_price,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions,
            }),
        }];
        let total_receipt_cost = Balance::from(gas + expected_gas_burnt) * gas_price;
        let expected_gas_burnt_amount = Balance::from(expected_gas_burnt) * GAS_PRICE;
        let expected_deficit = expected_gas_burnt_amount - total_receipt_cost;

        let result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts,
                &[],
            )
            .unwrap();
        // Used full prepaid gas, but it still not enough to cover deficit.
        assert_eq!(result.stats.gas_deficit_amount, expected_deficit);
        // Burnt all the fees + all prepaid gas.
        assert_eq!(result.stats.tx_burnt_amount, total_receipt_cost);
    }

    #[test]
    fn test_delete_key_add_key() {
        let initial_locked = to_yocto(500_000);
        let (runtime, tries, root, apply_state, signer) =
            setup_runtime(to_yocto(1_000_000), initial_locked, 10u64.pow(15));

        let state_update = tries.new_trie_update(root);
        let initial_account_state = get_account(&state_update, &alice_account()).unwrap().unwrap();

        let actions = vec![
            Action::DeleteKey(DeleteKeyAction { public_key: signer.public_key() }),
            Action::AddKey(AddKeyAction {
                public_key: signer.public_key(),
                access_key: AccessKey::full_access(),
            }),
        ];

        let receipts = create_receipts_with_actions(alice_account(), signer, actions);

        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts,
                &[],
            )
            .unwrap();
        let (store_update, root) =
            tries.apply_all(&apply_result.trie_changes, ).unwrap();
        store_update.commit().unwrap();

        let state_update = tries.new_trie_update(root);
        let final_account_state = get_account(&state_update, &alice_account()).unwrap().unwrap();

        assert_eq!(initial_account_state.storage_usage(), final_account_state.storage_usage());
    }

    #[test]
    fn test_delete_key_underflow() {
        let initial_locked = to_yocto(500_000);
        let (runtime, tries, root, apply_state, signer) =
            setup_runtime(to_yocto(1_000_000), initial_locked, 10u64.pow(15));

        let mut state_update = tries.new_trie_update(root);
        let mut initial_account_state =
            get_account(&state_update, &alice_account()).unwrap().unwrap();
        initial_account_state.set_storage_usage(10);
        set_account(&mut state_update, alice_account(), &initial_account_state);
        state_update.commit(StateChangeCause::InitialState);
        let trie_changes = state_update.finalize().unwrap().0;
        let (store_update, root) =
            tries.apply_all(&trie_changes, ).unwrap();
        store_update.commit().unwrap();

        let actions = vec![Action::DeleteKey(DeleteKeyAction { public_key: signer.public_key() })];

        let receipts = create_receipts_with_actions(alice_account(), signer, actions);

        let apply_result = runtime
            .apply(
                tries.get_trie(),
                root,
                &apply_state,
                &receipts,
                &[],
            )
            .unwrap();
        let (store_update, root) =
            tries.apply_all(&apply_result.trie_changes, ).unwrap();
        store_update.commit().unwrap();

        let state_update = tries.new_trie_update(root);
        let final_account_state = get_account(&state_update, &alice_account()).unwrap().unwrap();

        assert_eq!(final_account_state.storage_usage(), 0);
    }

    // #[test]
    // fn test_contract_precompilation() {
    //     let initial_balance = to_yocto(1_000_000);
    //     let initial_locked = to_yocto(500_000);
    //     let gas_limit = 10u64.pow(15);
    //     let (runtime, tries, root, apply_state, signer) =
    //         setup_runtime(initial_balance, initial_locked, gas_limit);

    //     let wasm_code = near_test_contracts::rs_contract().to_vec();
    //     let actions =
    //         vec![Action::DeployContract(DeployContractAction { code: wasm_code.clone() })];

    //     let receipts = create_receipts_with_actions(alice_account(), signer, actions);

    //     let apply_result = runtime
    //         .apply(
    //             tries.get_trie_for_shard(),
    //             root,
    //             &apply_state,
    //             &receipts,
    //             &[],
    //         )
    //         .unwrap();
    //     let (store_update, _) =
    //         tries.apply_all(&apply_result.trie_changes, ).unwrap();
    //     store_update.commit().unwrap();

    //     let contract_code = ContractCode::new(wasm_code);
    //     let vm_kind = VMKind::for_protocol_version(apply_state.current_protocol_version);
    //     let key = get_contract_cache_key(&contract_code, vm_kind, &apply_state.config.wasm_config);
    //     apply_state
    //         .cache
    //         .unwrap()
    //         .get(&key.0)
    //         .expect("Compiled contract should be cached")
    //         .expect("Compilation result should be non-empty");
    // }
}
