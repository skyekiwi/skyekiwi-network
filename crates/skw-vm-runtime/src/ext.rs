use std::sync::Arc;
use log::debug;

use skw_vm_primitives::errors::{StorageError};
use skw_vm_primitives::receipt::{ActionReceipt, DataReceiver, Receipt, ReceiptEnum};
use skw_vm_primitives::transaction::{
    Action, CreateAccountAction, DeleteAccountAction,
    DeployContractAction, FunctionCallAction, TransferAction,
};
use skw_vm_primitives::trie_key::{trie_key_parsers, TrieKey};
use skw_vm_primitives::contract_runtime::{AccountId, Balance, CryptoHash, ContractCode};
use skw_vm_primitives::utils::create_data_id;
use skw_vm_store::{get_code, TrieUpdate, TrieUpdateValuePtr};
use skw_vm_primitives::errors::{HostError, VMLogicError};
use skw_vm_host::{RuntimeExternal as External, ValuePtr};

pub struct RuntimeExt<'a> {
    trie_update: &'a mut TrieUpdate,
    account_id: &'a AccountId,
    action_receipts: Vec<(AccountId, ActionReceipt)>,
    signer_id: &'a AccountId,
    gas_price: Balance,
    action_hash: &'a CryptoHash,
    data_count: u64,
}

/// Error used by `RuntimeExt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExternalError {
    /// Unexpected error which is typically related to the node storage corruption.
    /// It's possible the input state is invalid or malicious.
    StorageError(StorageError),
}

impl From<ExternalError> for VMLogicError {
    fn from(_: ExternalError) -> Self {
        // let ExternalError(v) = err;
        VMLogicError::ExternalError(b"external error".to_vec())
    }
}

pub struct RuntimeExtValuePtr<'a>(TrieUpdateValuePtr<'a>);

impl<'a> ValuePtr for RuntimeExtValuePtr<'a> {
    fn len(&self) -> u32 {
        self.0.len()
    }

    fn deref(&self) -> ExtResult<Vec<u8>> {
        self.0.deref_value().map_err(wrap_storage_error)
    }
}

impl<'a> RuntimeExt<'a> {
    pub fn new(
        trie_update: &'a mut TrieUpdate,
        account_id: &'a AccountId,
        signer_id: &'a AccountId,
        gas_price: Balance,
        action_hash: &'a CryptoHash,
    ) -> Self {
        RuntimeExt {
            trie_update,
            account_id,
            action_receipts: vec![],
            signer_id,
            gas_price,
            action_hash,
            data_count: 0,
        }
    }

    #[inline]
    pub fn account_id(&self) -> &'a AccountId {
        self.account_id
    }

    pub fn get_code(
        &self,
        code_hash: CryptoHash,
    ) -> Result<Option<Arc<ContractCode>>, StorageError> {
        debug!(target:"runtime", "Calling the contract at account {}", self.account_id);
        let code = || get_code(self.trie_update, self.account_id, Some(code_hash));
        crate::cache::get_code(code_hash, code)
    }

    pub fn create_storage_key(&self, key: &[u8]) -> TrieKey {
        TrieKey::ContractData { account_id: self.account_id.clone(), key: key.to_vec() }
    }

    fn new_data_id(&mut self) -> CryptoHash {
        let data_id = create_data_id(
            self.action_hash,
            self.data_count as usize,
        );
        self.data_count += 1;
        data_id
    }

    pub fn into_receipts(self, predecessor_id: &AccountId) -> Vec<Receipt> {
        self.action_receipts
            .into_iter()
            .map(|(receiver_id, action_receipt)| Receipt {
                predecessor_id: predecessor_id.clone(),
                receiver_id,
                // Actual receipt ID is set in the Runtime.apply_action_receipt(...) in the
                // "Generating receipt IDs" section
                receipt_id: CryptoHash::default(),
                receipt: ReceiptEnum::Action(action_receipt),
            })
            .collect()
    }

    fn append_action(&mut self, receipt_index: u64, action: Action) {
        self.action_receipts
            .get_mut(receipt_index as usize)
            .expect("receipt index should be present")
            .1
            .actions
            .push(action);
    }
}

fn wrap_storage_error(error: StorageError) -> VMLogicError {
    VMLogicError::from(ExternalError::StorageError(error))
}

type ExtResult<T> = ::std::result::Result<T, VMLogicError>;

impl<'a> External for RuntimeExt<'a> {
    fn storage_set(&mut self, key: &[u8], value: &[u8]) -> ExtResult<()> {
        let storage_key = self.create_storage_key(key);
        self.trie_update.set(storage_key, Vec::from(value));
        Ok(())
    }

    fn storage_get<'b>(&'b self, key: &[u8]) -> ExtResult<Option<Box<dyn ValuePtr + 'b>>> {
        let storage_key = self.create_storage_key(key);
        self.trie_update
            .get_ref(&storage_key)
            .map_err(wrap_storage_error)
            .map(|option| option.map(|ptr| Box::new(RuntimeExtValuePtr(ptr)) as Box<_>))
    }

    fn storage_remove(&mut self, key: &[u8]) -> ExtResult<()> {
        let storage_key = self.create_storage_key(key);
        self.trie_update.remove(storage_key);
        Ok(())
    }

    fn storage_has_key(&mut self, key: &[u8]) -> ExtResult<bool> {
        let storage_key = self.create_storage_key(key);
        self.trie_update.get_ref(&storage_key).map(|x| x.is_some()).map_err(wrap_storage_error)
    }

    fn storage_remove_subtree(&mut self, prefix: &[u8]) -> ExtResult<()> {
        let data_keys = self
            .trie_update
            .iter(&trie_key_parsers::get_raw_prefix_for_contract_data(self.account_id, prefix))
            .map_err(wrap_storage_error)?
            .map(|raw_key| {
                trie_key_parsers::parse_data_key_from_contract_data_key(&raw_key?, self.account_id)
                    .map_err(|_e| {
                        StorageError::StorageInconsistentState(
                            "Can't parse data key from raw key for ContractData".to_string(),
                        )
                    })
                    .map(Vec::from)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(wrap_storage_error)?;
        for key in data_keys {
            self.trie_update
                .remove(TrieKey::ContractData { account_id: self.account_id.clone(), key });
        }
        Ok(())
    }

    fn create_receipt(
        &mut self,
        receipt_indices: Vec<u64>,
        receiver_id: AccountId,
    ) -> ExtResult<u64> {
        let mut input_data_ids = vec![];
        for receipt_index in receipt_indices {
            let data_id = self.new_data_id();
            self.action_receipts
                .get_mut(receipt_index as usize)
                .ok_or_else(|| HostError::InvalidReceiptIndex { receipt_index })?
                .1
                .output_data_receivers
                .push(DataReceiver { data_id, receiver_id: receiver_id.clone() });
            input_data_ids.push(data_id);
        }

        let new_receipt = ActionReceipt {
            signer_id: self.signer_id.clone(),
            gas_price: self.gas_price,
            output_data_receivers: vec![],
            input_data_ids,
            actions: vec![],
        };
        let new_receipt_index = self.action_receipts.len() as u64;
        self.action_receipts.push((receiver_id, new_receipt));
        Ok(new_receipt_index)
    }

    fn append_action_create_account(&mut self, receipt_index: u64) -> ExtResult<()> {
        self.append_action(receipt_index, Action::CreateAccount(CreateAccountAction {}));
        Ok(())
    }

    fn append_action_transfer(&mut self, receipt_index: u64, deposit: u128) -> ExtResult<()> {
        self.append_action(receipt_index, Action::Transfer(TransferAction { deposit }));
        Ok(())
    }

    fn append_action_delete_account(
        &mut self,
        receipt_index: u64,
        beneficiary_id: AccountId,
    ) -> ExtResult<()> {
        self.append_action(
            receipt_index,
            Action::DeleteAccount(DeleteAccountAction { beneficiary_id }),
        );
        Ok(())
    }

    fn append_action_deploy_contract(
        &mut self,
        receipt_index: u64,
        code: Vec<u8>,
    ) -> ExtResult<()> {
        self.append_action(receipt_index, Action::DeployContract(DeployContractAction { code }));
        Ok(())
    }

    fn append_action_function_call(
        &mut self,
        receipt_index: u64,
        method_name: Vec<u8>,
        args: Vec<u8>,
        attached_deposit: u128,
        prepaid_gas: u64,
    ) -> ExtResult<()> {
        self.append_action(
            receipt_index,
            Action::FunctionCall(FunctionCallAction {
                method_name: String::from_utf8(method_name)
                    .map_err(|_| HostError::InvalidMethodName)?,
                args,
                gas: prepaid_gas,
                deposit: attached_deposit,
            }),
        );
        Ok(())
    }

    fn get_touched_nodes_count(&self) -> u64 {
        self.trie_update.trie.counter.get()
    }

    // TODO: look into this
    fn reset_touched_nodes_counter(&mut self) {

    }
}
