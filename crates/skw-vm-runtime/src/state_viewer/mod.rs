use crate::{actions::execute_function_call, ext::RuntimeExt};
use log::debug;

use skw_vm_primitives::config::RuntimeConfig;
use skw_vm_primitives::{
    receipt::ActionReceipt,
    apply_state::ApplyState,
    serialize::to_base64,
    transaction::FunctionCallAction,
    trie_key::trie_key_parsers,
    account::{Account},
    contract_runtime::{AccountId, Gas, CryptoHash, ContractCode},
    views::{StateItem, ViewApplyState, ViewStateResult},
};

use skw_vm_store::{get_account, get_code, TrieUpdate};
use skw_vm_host::{ReturnData, ViewConfig};
use std::{str, time::Instant};
use std::sync::Arc;
pub mod errors;

pub struct TrieViewer {
    /// Upper bound of the byte size of contract state that is still viewable. None is no limit
    state_size_limit: Option<u64>,
    /// Gas limit used when when handling call_function queries.
    max_gas_burnt_view: Gas,
}

impl Default for TrieViewer {
    fn default() -> Self {
        // let config_store = RuntimeConfigStore::new(None);
        // let latest_runtime_config = config_store.get_config(PROTOCOL_VERSION);
        let runtime_config = RuntimeConfig::test();
        let max_gas_burnt = runtime_config.wasm_config.limit_config.max_gas_burnt;
        Self { state_size_limit: None, max_gas_burnt_view: max_gas_burnt }
    }
}

impl TrieViewer {
    pub fn new(state_size_limit: Option<u64>, max_gas_burnt_view: Option<Gas>) -> Self {
        let max_gas_burnt_view =
            max_gas_burnt_view.unwrap_or_else(|| TrieViewer::default().max_gas_burnt_view);
        Self { state_size_limit, max_gas_burnt_view }
    }

    pub fn view_account(
        &self,
        state_update: &TrieUpdate,
        account_id: &AccountId,
    ) -> Result<Account, errors::ViewAccountError> {
        get_account(state_update, account_id)?.ok_or_else(|| {
            errors::ViewAccountError::AccountDoesNotExist {
                requested_account_id: account_id.clone(),
            }
        })
    }

    pub fn view_contract_code(
        &self,
        state_update: &TrieUpdate,
        account_id: &AccountId,
    ) -> Result<ContractCode, errors::ViewContractCodeError> {
        let account = self.view_account(state_update, account_id)?;
        get_code(state_update, account_id, Some(account.code_hash()))?.ok_or_else(|| {
            errors::ViewContractCodeError::NoContractCode {
                contract_account_id: account_id.clone(),
            }
        })
    }

    pub fn view_state(
        &self,
        state_update: &TrieUpdate,
        account_id: &AccountId,
        prefix: &[u8],
    ) -> Result<ViewStateResult, errors::ViewStateError> {
        match get_account(state_update, account_id)? {
            Some(account) => {
                let code_len = get_code(state_update, account_id, Some(account.code_hash()))?
                    .map(|c| c.code.len() as u64)
                    .unwrap_or_default();
                if let Some(limit) = self.state_size_limit {
                    if account.storage_usage().saturating_sub(code_len) > limit {
                        return Err(errors::ViewStateError::AccountStateTooLarge {
                            requested_account_id: account_id.clone(),
                        });
                    }
                }
            }
            None => {
                return Err(errors::ViewStateError::AccountDoesNotExist {
                    requested_account_id: account_id.clone(),
                })
            }
        };

        let mut values = vec![];
        let query = trie_key_parsers::get_raw_prefix_for_contract_data(account_id, prefix);
        let acc_sep_len = query.len() - prefix.len();
        let mut iter = state_update.trie.iter(&state_update.get_root())?;
        iter.seek(&query)?;
        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(query.as_ref()) {
                break;
            }
            values.push(StateItem {
                key: to_base64(&key[acc_sep_len..]),
                value: to_base64(&value),
                proof: vec![],
            });
        }
        // TODO(2076): Add proofs for the storage items.
        Ok(ViewStateResult { values, proof: vec![] })
    }

    pub fn call_function(
        &self,
        mut state_update: TrieUpdate,
        view_state: ViewApplyState,
        contract_id: &AccountId,
        method_name: &str,
        args: &[u8],
        logs: &mut Vec<String>,
    ) -> Result<Vec<u8>, errors::CallFunctionError> {
        let now = Instant::now();
        let root = state_update.get_root();
        let mut account = get_account(&state_update, contract_id)?.ok_or_else(|| {
            errors::CallFunctionError::AccountDoesNotExist {
                requested_account_id: contract_id.clone(),
            }
        })?;
        // TODO(#1015): Add ability to pass public key and originator_id
        let originator_id = contract_id;
        let empty_hash = CryptoHash::default();
        let mut runtime_ext = RuntimeExt::new(
            &mut state_update,
            contract_id,
            originator_id,
            0,
            &empty_hash,
        );
        
        let config = RuntimeConfig::test();
        let apply_state = ApplyState {
            block_number: view_state.block_number,
            // Used for legacy reasons
            prev_block_hash: view_state.prev_block_hash,
            block_hash: view_state.block_hash,
            gas_price: 0,
            block_timestamp: view_state.block_timestamp,
            gas_limit: None,
            random_seed: root,
            config: Arc::new(config.clone()),
        };

        let action_receipt = ActionReceipt {
            signer_id: originator_id.clone(),
            gas_price: 0,
            output_data_receivers: vec![],
            input_data_ids: vec![],
            actions: vec![],
        };
        let function_call = FunctionCallAction {
            method_name: method_name.to_string(),
            args: args.to_vec(),
            gas: self.max_gas_burnt_view,
            deposit: 0,
        };
        let (outcome, err) = execute_function_call(
            &apply_state,
            &mut runtime_ext,
            &mut account,
            originator_id,
            &action_receipt,
            &[],
            &function_call,
            &empty_hash,
            &config,
            true,
            Some(ViewConfig { max_gas_burnt: self.max_gas_burnt_view }),
        );
        let elapsed = now.elapsed();
        let time_ms =
            (elapsed.as_secs() as f64 / 1_000.0) + f64::from(elapsed.subsec_nanos()) / 1_000_000.0;
        let time_str = format!("{:.*}ms", 2, time_ms);

        if let Some(err) = err {
            if let Some(outcome) = outcome {
                logs.extend(outcome.logs);
            }
            let message = format!("wasm execution failed with error: {:?}", err);
            debug!(target: "runtime", "(exec time {}) {}", time_str, message);
            Err(errors::CallFunctionError::VMError { error_message: message })
        } else {
            let outcome = outcome.unwrap();
            debug!(target: "runtime", "(exec time {}) result of execution: {:?}", time_str, outcome);
            logs.extend(outcome.logs);
            let result = match outcome.return_data {
                ReturnData::Value(buf) => buf,
                ReturnData::ReceiptIndex(_) | ReturnData::None => vec![],
            };
            Ok(result)
        }
    }
}
