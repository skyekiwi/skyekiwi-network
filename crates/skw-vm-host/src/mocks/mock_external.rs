use crate::{RuntimeExternal, ValuePtr};
use skw_vm_primitives::contract_runtime::{AccountId, Balance, Gas};
use skw_vm_primitives::errors::HostError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Clone)]
/// Emulates the trie and the mock handling code.
pub struct MockedExternal {
    pub fake_trie: HashMap<Vec<u8>, Vec<u8>>,
    receipts: Vec<Receipt>,
    pub validators: HashMap<AccountId, Balance>,
}

pub struct MockedValuePtr {
    value: Vec<u8>,
}

impl MockedValuePtr {
    pub fn new<T>(value: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        MockedValuePtr { value: value.as_ref().to_vec() }
    }
}

impl ValuePtr for MockedValuePtr {
    fn len(&self) -> u32 {
        self.value.len() as u32
    }

    fn deref(&self) -> crate::dependencies::Result<Vec<u8>> {
        Ok(self.value.clone())
    }
}

impl MockedExternal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get calls to receipt create that were performed during contract call.
    pub fn get_receipt_create_calls(&self) -> &Vec<Receipt> {
        &self.receipts
    }
}

use crate::dependencies::Result;
impl RuntimeExternal for MockedExternal {
    fn storage_set(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.fake_trie.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn storage_get(&self, key: &[u8]) -> Result<Option<Box<dyn ValuePtr>>> {
        Ok(self
            .fake_trie
            .get(key)
            .map(|value| Box::new(MockedValuePtr { value: value.clone() }) as Box<_>))
    }

    fn storage_remove(&mut self, key: &[u8]) -> Result<()> {
        self.fake_trie.remove(key);
        Ok(())
    }

    fn storage_remove_subtree(&mut self, prefix: &[u8]) -> Result<()> {
        self.fake_trie.retain(|key, _| !key.starts_with(prefix));
        Ok(())
    }

    fn storage_has_key(&mut self, key: &[u8]) -> Result<bool> {
        Ok(self.fake_trie.contains_key(key))
    }

    fn create_receipt(&mut self, receipt_indices: Vec<u64>, receiver_id: AccountId) -> Result<u64> {
        if let Some(index) = receipt_indices.iter().find(|&&el| el >= self.receipts.len() as u64) {
            return Err(HostError::InvalidReceiptIndex { receipt_index: *index }.into());
        }
        let res = self.receipts.len() as u64;
        self.receipts.push(Receipt { receipt_indices, receiver_id, actions: vec![] });
        Ok(res)
    }

    fn append_action_create_account(&mut self, receipt_index: u64) -> Result<()> {
        self.receipts.get_mut(receipt_index as usize).unwrap().actions.push(Action::CreateAccount);
        Ok(())
    }

    fn append_action_deploy_contract(&mut self, receipt_index: u64, code: Vec<u8>) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::DeployContract(DeployContractAction { code }));
        Ok(())
    }

    fn append_action_function_call(
        &mut self,
        receipt_index: u64,
        method_name: Vec<u8>,
        arguments: Vec<u8>,
        attached_deposit: u128,
        prepaid_gas: u64,
    ) -> Result<()> {
        self.receipts.get_mut(receipt_index as usize).unwrap().actions.push(Action::FunctionCall(
            FunctionCallAction {
                method_name,
                args: arguments,
                deposit: attached_deposit,
                gas: prepaid_gas,
            },
        ));
        Ok(())
    }

    fn append_action_transfer(&mut self, receipt_index: u64, amount: u128) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::Transfer(TransferAction { deposit: amount }));
        Ok(())
    }

    fn append_action_delete_account(
        &mut self,
        receipt_index: u64,
        beneficiary_id: AccountId,
    ) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::DeleteAccount(DeleteAccountAction { beneficiary_id }));
        Ok(())
    }

    fn get_touched_nodes_count(&self) -> u64 {
        0
    }

    fn reset_touched_nodes_counter(&mut self) {}
}



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Receipt {
    receipt_indices: Vec<u64>,
    receiver_id: AccountId,
    actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Action {
    CreateAccount,
    DeployContract(DeployContractAction),
    FunctionCall(FunctionCallAction),
    Transfer(TransferAction),
    DeleteAccount(DeleteAccountAction),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeployContractAction {
    pub code: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCallAction {
    #[serde(with = "crate::serde_with::bytes_as_str")]
    method_name: Vec<u8>,
    /// Most function calls still take JSON as input, so we'll keep it there as a string.
    /// Once we switch to borsh, we'll have to switch to base64 encoding.
    /// Right now, it is only used with standalone runtime when passing in Receipts or expecting
    /// receipts. The workaround for input is to use a VMContext input.
    #[serde(with = "crate::serde_with::bytes_as_str")]
    args: Vec<u8>,
    gas: Gas,
    deposit: Balance,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferAction {
    deposit: Balance,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteAccountAction {
    beneficiary_id: AccountId,
}
