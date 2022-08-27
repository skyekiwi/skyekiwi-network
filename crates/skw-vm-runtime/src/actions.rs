use skw_vm_primitives::errors::{ActionErrorKind, ContractCallError, RuntimeError, ActionError};
use skw_vm_primitives::contract_runtime::{CryptoHash, ContractCode, AccountId};
use skw_vm_primitives::receipt::{ActionReceipt, Receipt};
use skw_vm_primitives::fees::{RuntimeFeesConfig};
// use skw_vm_primitives::config::{AccountCreationConfig};

use skw_vm_primitives::account::{Account};
use skw_vm_primitives::transaction::{
    Action, DeleteAccountAction, DeployContractAction,
    FunctionCallAction, TransferAction,
};
use skw_vm_primitives::utils::create_random_seed;

use skw_vm_store::{
    get_code, remove_account, set_code,
    StorageError, TrieUpdate,
};
use skw_vm_primitives::errors::{
    CacheError, CompilationError, FunctionCallError, VMError,
};
use skw_vm_host::types::PromiseResult;
use skw_vm_host::{VMContext, VMOutcome};

use crate::config::{safe_add_gas, RuntimeConfig};
use crate::ext::{RuntimeExt};
use crate::{ActionResult, ApplyState};
use skw_vm_primitives::config::ViewConfig;

// use skw_vm_engine::precompile_contract;

/// Runs given function call with given context / apply state.
pub(crate) fn execute_function_call(
    apply_state: &ApplyState,
    runtime_ext: &mut RuntimeExt,
    account: &mut Account,
    predecessor_id: &AccountId,
    action_receipt: &ActionReceipt,
    promise_results: &[PromiseResult],
    function_call: &FunctionCallAction,
    action_hash: &CryptoHash,
    config: &RuntimeConfig,
    is_last_action: bool,
    view_config: Option<ViewConfig>,
) -> (Option<VMOutcome>, Option<VMError>) {
    let account_id = runtime_ext.account_id();

    let code = match runtime_ext.get_code(account.code_hash()) {
        Ok(Some(code)) => code,
        Ok(None) => {
            let error = FunctionCallError::CompilationError(CompilationError::CodeDoesNotExist {
                account_id: account_id.clone(),
            });
            return (None, Some(VMError::FunctionCallError(error)));
        }
        Err(_e) => {
            return (
                None,
                Some(VMError::ExternalError(b"storage error".to_vec()))
                // TODO: unwrap might fail
                //Some(VMError::ExternalError(StorageError(e).try_to_vec().unwrap())),
            );
        }
    };
    // Output data receipts are ignored if the function call is not the last action in the batch.
    let output_data_receivers: Vec<_> = if is_last_action {
        action_receipt.output_data_receivers.iter().map(|r| r.receiver_id.clone()).collect()
    } else {
        vec![]
    };
    let random_seed = create_random_seed(
        *action_hash,
        apply_state.random_seed,
    );
    let context = VMContext {
        current_account_id: runtime_ext.account_id().clone(),
        signer_account_id: action_receipt.signer_id.clone(),
        predecessor_account_id: predecessor_id.clone(),
        input: function_call.args.clone(),
        block_number: apply_state.block_number,
        block_timestamp: apply_state.block_timestamp,
        account_balance: account.amount(),
        storage_usage: account.storage_usage(),
        attached_deposit: function_call.deposit,
        prepaid_gas: function_call.gas,
        random_seed,
        view_config,
        output_data_receivers,
    };

    skw_vm_engine::WasmiVM::run(
        &code,
        &function_call.method_name,
        runtime_ext,
        context,
        &config.wasm_config,
        &config.transaction_costs,
        promise_results,
    )
}

pub(crate) fn action_transfer(
    account: &mut Account,
    transfer: &TransferAction,
) -> Result<(), StorageError> {
    account.set_amount(account.amount().checked_add(transfer.deposit).ok_or_else(|| {
        StorageError::StorageInconsistentState("Account balance integer overflow".to_string())
    })?);
    Ok(())
}

pub(crate) fn action_create_account(
    fee_config: &RuntimeFeesConfig,
    account: &mut Option<Account>,
    actor_id: &mut AccountId,
    account_id: &AccountId,
    // This nenver fails
    _result: &mut ActionResult,
) {
    *actor_id = account_id.clone();
    *account = Some(Account::new(
        0,
        0,
        CryptoHash::default(),
        fee_config.storage_usage_config.num_bytes_account,
        0,
    ));
}

pub(crate) fn action_delete_account(
    state_update: &mut TrieUpdate,
    account: &mut Option<Account>,
    actor_id: &mut AccountId,
    receipt: &Receipt,
    result: &mut ActionResult,
    account_id: &AccountId,
    delete_account: &DeleteAccountAction,
) -> Result<(), StorageError> {

    {
        let account = account.as_ref().unwrap();
        let mut account_storage_usage = account.storage_usage();
        let contract_code = get_code(state_update, account_id, Some(account.code_hash()))?;
        if let Some(code) = contract_code {
            // account storage usage should be larger than code size
            let code_len = code.code.len() as u64;
            debug_assert!(account_storage_usage > code_len);
            account_storage_usage = account_storage_usage.saturating_sub(code_len);
        }
        if account_storage_usage > Account::MAX_ACCOUNT_DELETION_STORAGE_USAGE {
            result.result = Err(ActionErrorKind::DeleteAccountWithLargeState {
                account_id: account_id.clone(),
            }
            .into());
            return Ok(());
        }
    }
    

    // We use current amount as a pay out to beneficiary.
    let account_balance = account.as_ref().unwrap().amount();
    if account_balance > 0 {
        result
            .new_receipts
            .push(Receipt::new_force_transfer(&delete_account.beneficiary_id, account_balance));
    }
    remove_account(state_update, account_id)?;
    *actor_id = receipt.predecessor_id.clone();
    *account = None;
    Ok(())
}

pub(crate) fn check_actor_permissions(
    action: &Action,
    account: &Option<Account>,
    actor_id: &AccountId,
    account_id: &AccountId,
) -> Result<(), ActionError> {
    match action {
        Action::DeployContract(_) => {
            if actor_id != account_id {
                return Err(ActionErrorKind::ActorNoPermission {
                    account_id: account_id.clone(),
                    actor_id: actor_id.clone(),
                }
                .into());
            }
        }
        Action::DeleteAccount(_) => {
            if actor_id != account_id {
                return Err(ActionErrorKind::ActorNoPermission {
                    account_id: account_id.clone(),
                    actor_id: actor_id.clone(),
                }
                .into());
            }
            let account = account.as_ref().unwrap();
            if account.locked() != 0 {
                return Err(ActionErrorKind::DeleteAccountStaking {
                    account_id: account_id.clone(),
                }
                .into());
            }
        }
        Action::CreateAccount(_) | Action::FunctionCall(_) | Action::Transfer(_) => (),
    };
    Ok(())
}

pub(crate) fn check_account_existence(
    action: &Action,
    account: &mut Option<Account>,
    account_id: &AccountId,
) -> Result<(), ActionError> {
    match action {
        Action::CreateAccount(_) => {
            if account.is_some() {
                return Err(ActionErrorKind::AccountAlreadyExists {
                    account_id: account_id.clone(),
                }
                .into());
            } 
        }
        Action::Transfer(_) => {
            if account.is_none() {
                return Err(ActionErrorKind::AccountDoesNotExist { account_id: account_id.clone() }
                    .into())
            }
        }
        Action::DeployContract(_)
        | Action::FunctionCall(_)
        | Action::DeleteAccount(_) => {
            if account.is_none() {
                return Err(ActionErrorKind::AccountDoesNotExist {
                    account_id: account_id.clone(),
                }
                .into());
            }
        }
    };
    Ok(())
}

// TODO: review this
pub(crate) fn action_function_call(
    state_update: &mut TrieUpdate,
    apply_state: &ApplyState,
    account: &mut Account,
    receipt: &Receipt,
    action_receipt: &ActionReceipt,
    promise_results: &[PromiseResult],
    result: &mut ActionResult,
    account_id: &AccountId,
    function_call: &FunctionCallAction,
    action_hash: &CryptoHash,
    config: &RuntimeConfig,
    is_last_action: bool,
) -> Result<(), RuntimeError> {

    if account.amount().checked_add(function_call.deposit).is_none() {
        return Err(StorageError::StorageInconsistentState(
            "Account balance integer overflow during function call deposit".to_string(),
        )
        .into());
    }

    let mut runtime_ext = RuntimeExt::new(
        state_update,
        account_id,
        &action_receipt.signer_id,
        action_receipt.gas_price,
        action_hash,
    );

    let (outcome, err) = execute_function_call(
        apply_state,
        &mut runtime_ext,
        account,
        &receipt.predecessor_id,
        action_receipt,
        promise_results,
        function_call,
        action_hash,
        config,
        is_last_action,
        None,
    );

    let execution_succeeded = match err {
        Some(VMError::FunctionCallError(er)) => match er {
            FunctionCallError::Nondeterministic(msg) => {
                panic!("Contract runner returned non-deterministic error '{}', aborting", msg)
            }
            FunctionCallError::WasmUnknownError { debug_message } => {
                panic!("Wasmer returned unknown message: {}", debug_message)
            }
            FunctionCallError::CompilationError(err) => {
                result.result = Err(ActionErrorKind::FunctionCallError(
                    ContractCallError::CompilationError(err).into(),
                )
                .into());
                false
            }
            // TODO: we don't really have linkError - change this!
            FunctionCallError::LinkError { msg } => {
                result.result = Err(ActionErrorKind::FunctionCallError(
                    ContractCallError::ExecutionError { msg: format!("Link Error: {}", msg) }
                        .into(),
                )
                .into());
                false
            }
            FunctionCallError::MethodResolveError(err) => {
                result.result = Err(ActionErrorKind::FunctionCallError(
                    ContractCallError::MethodResolveError(err).into(),
                )
                .into());
                false
            }
            FunctionCallError::WasmTrap(_) | FunctionCallError::HostError(_) => {
                result.result = Err(ActionErrorKind::FunctionCallError(
                    ContractCallError::ExecutionError { msg: er.to_string() }.into(),
                )
                .into());
                false
            }
            FunctionCallError::_EVMError => unreachable!(),
        },
        Some(VMError::ExternalError(_)) => {
            // TODO: look into this
            return Err(RuntimeError::StorageError);//(StorageInternalError)
        }
        Some(VMError::InconsistentStateError(e)) => {
            return Err(StorageError::StorageInconsistentState(e.to_string()).into());
        }
        Some(VMError::CacheError(e)) => {
            let message = match e {
                CacheError::DeserializationError => "Cache deserialization error",
                CacheError::SerializationError { hash: _hash } => "Cache serialization error",
                CacheError::ReadError => "Cache read error",
                CacheError::WriteError => "Cache write error",
            };
            return Err(StorageError::StorageInconsistentState(message.to_string()).into());
        }
        None => true,
    };
    
    if let Some(outcome) = outcome {

        // NOTE: translate VMOutcome to ActionResult
        result.gas_burnt = safe_add_gas(result.gas_burnt, outcome.burnt_gas)?;
        result.gas_burnt_for_function_call =
            safe_add_gas(result.gas_burnt_for_function_call, outcome.burnt_gas)?;
        // Runtime in `generate_refund_receipts` takes care of using proper value for refunds.
        // It uses `gas_used` for success and `gas_burnt` for failures. So it's not an issue to
        // return a real `gas_used` instead of the `gas_burnt` into `ActionResult` even for
        // `FunctionCall`s error.
        result.gas_used = safe_add_gas(result.gas_used, outcome.used_gas)?;
        
        // Note: logs are events - needs to be serde'd 
        result.logs.extend(outcome.logs.into_iter());

        result.profile.merge(&outcome.profile);
        if execution_succeeded {
            account.set_amount(outcome.balance);
            account.set_storage_usage(outcome.storage_usage);

            result.result = Ok(outcome.return_data);
            
            // TODO: link to into_receipts(account: id) ??
            result.new_receipts.extend(runtime_ext.into_receipts(account_id));
        }
    } else {
        assert!(!execution_succeeded, "Outcome should always be available if execution succeeded")
    }
    Ok(())
}

pub(crate) fn action_deploy_contract(
    state_update: &mut TrieUpdate,
    account_id: &AccountId,
    account: &mut Account,
    deploy_contract: &DeployContractAction,
    apply_state: &ApplyState,
) -> Result<(), StorageError> {
    let code = ContractCode::new(&deploy_contract.code);
    let prev_code = get_code(state_update, account_id, Some(account.code_hash()))?;
    let prev_code_length = prev_code.map(|code| code.code.len() as u64).unwrap_or_default();
    
    account.set_storage_usage(account.storage_usage().saturating_sub(prev_code_length));
    account.set_storage_usage(
        account.storage_usage().checked_add(code.code.len() as u64).ok_or_else(|| {
            StorageError::StorageInconsistentState(format!(
                "Storage usage integer overflow for account {}",
                account_id
            ))
        })?,
    );
    account.set_code_hash(code.hash);
    set_code(state_update, account_id.clone(), &code);
    
    // Precompile the contract and store result (compiled code or error) in the database.
    // Note, that contract compilation costs are already accounted in deploy cost using
    // special logic in estimator (see get_runtime_config() function).
    skw_vm_engine::create_module_instance(
        &code,
        &apply_state.config.wasm_config,
        skw_vm_engine::WasmiMemory::new(
            apply_state.config.wasm_config.limit_config.initial_memory_pages,
            apply_state.config.wasm_config.limit_config.max_memory_pages,
        ).expect("Cannot create memory for a contract call").clone()
    )
    .ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use skw_vm_primitives::contract_runtime::hash_bytes;
    use skw_vm_primitives::trie_key::TrieKey;
    use skw_vm_store::test_utils::create_tries;
    use skw_vm_primitives::errors::ActionError;
    use super::*;

    #[test]
    fn test_create_account() {
        let account_id = AccountId::test();
        let mut actor_id = AccountId::system();
        let mut account = None;

        let mut action_result = ActionResult::default();
        action_create_account(
            &RuntimeFeesConfig::test(),
            &mut account,
            &mut actor_id,
            &account_id,
            &mut action_result,
        );

        assert!(account.is_some());
        assert_eq!(actor_id, account_id);
        assert!(action_result.result.is_ok());
    }

    fn test_delete_large_account(
        account_id: &AccountId,
        code_hash: &CryptoHash,
        storage_usage: u64,
        state_update: &mut TrieUpdate,
    ) -> ActionResult {
        let mut account = Some(Account::new(100, 0, *code_hash, storage_usage, 0u64));
        let mut actor_id = account_id.clone();
        let mut action_result = ActionResult::default();
        let receipt = Receipt::new_force_transfer(&AccountId::system(), 0);
        let res = action_delete_account(
            state_update,
            &mut account,
            &mut actor_id,
            &receipt,
            &mut action_result,
            account_id,
            &DeleteAccountAction { beneficiary_id: AccountId::test() },
        );
        assert!(res.is_ok());
        action_result
    }

    #[test]
    fn test_delete_account_too_large() {
        let tries = create_tries();
        let mut state_update =
            tries.new_trie_update([0; 32]);
        let action_result = test_delete_large_account(
            &AccountId::system(),
            &[0; 32],
            Account::MAX_ACCOUNT_DELETION_STORAGE_USAGE + 1,
            &mut state_update,
        );
        assert_eq!(
            action_result.result,
            Err(ActionError {
                index: None,
                kind: ActionErrorKind::DeleteAccountWithLargeState {
                    account_id: AccountId::system()
                }
            })
        )
    }

    fn test_delete_account_with_contract(storage_usage: u64) -> ActionResult {
        let tries = create_tries();
        let mut state_update =
            tries.new_trie_update( CryptoHash::default());
        let account_id = AccountId::system();
        let trie_key = TrieKey::ContractCode { account_id: account_id.clone() };
        let empty_contract = [0; 10_000].to_vec();
        let contract_hash = hash_bytes(&empty_contract);
        state_update.set(trie_key, empty_contract);
        test_delete_large_account(&account_id, &contract_hash, storage_usage, &mut state_update)
    }

    #[test]
    fn test_delete_account_with_contract_and_small_state() {
        let action_result =
            test_delete_account_with_contract(Account::MAX_ACCOUNT_DELETION_STORAGE_USAGE + 100);
        assert!(action_result.result.is_ok());
    }

    #[test]
    fn test_delete_account_with_contract_and_large_state() {
        let action_result =
            test_delete_account_with_contract(10 * Account::MAX_ACCOUNT_DELETION_STORAGE_USAGE);
        assert_eq!(
            action_result.result,
            Err(ActionError {
                index: None,
                kind: ActionErrorKind::DeleteAccountWithLargeState {
                    account_id: AccountId::system()
                }
            })
        );
    }
}
