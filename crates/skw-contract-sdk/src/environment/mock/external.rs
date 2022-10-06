use super::{Receipt, VmAction};
use crate::{Balance};

use skw_vm_host::types::AccountId as VmAccountId;
use skw_vm_host::{RuntimeExternal as External, HostError, ValuePtr};
use std::{ collections::HashMap };

type Result<T> = ::core::result::Result<T, skw_vm_host::VMLogicError>;

#[derive(Default, Clone)]
/// Emulates the trie and the mock handling code for the SDK. This is a modified version of
/// `MockedExternal` from `skw_vm_host`.
pub(crate) struct SdkExternal {
    pub fake_trie: HashMap<Vec<u8>, Vec<u8>>,
    pub receipts: Vec<Receipt>,
    pub validators: HashMap<String, Balance>,
}

pub struct MockedValuePtr {
    value: Vec<u8>,
}

impl ValuePtr for MockedValuePtr {
    fn len(&self) -> u32 {
        self.value.len() as u32
    }

    fn deref(&self) -> Result<Vec<u8>> {
        Ok(self.value.clone())
    }
}

impl SdkExternal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl External for SdkExternal {
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

    fn create_receipt(
        &mut self,
        receipt_indices: Vec<u64>,
        receiver_id: VmAccountId,
    ) -> Result<u64> {
        if let Some(index) = receipt_indices.iter().find(|&&el| el >= self.receipts.len() as u64) {
            return Err(HostError::InvalidReceiptIndex { receipt_index: *index }.into());
        }
        let res = self.receipts.len() as u64;
        self.receipts.push(Receipt {
            receipt_indices,
            receiver_id: receiver_id.into(),
            actions: vec![],
        });
        Ok(res)
    }

    fn append_action_create_account(&mut self, receipt_index: u64) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(VmAction::CreateAccount);
        Ok(())
    }

    fn append_action_deploy_contract(&mut self, receipt_index: u64, code: Vec<u8>) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(VmAction::DeployContract { code });
        Ok(())
    }

    fn append_action_function_call(
        &mut self,
        receipt_index: u64,
        function_name: Vec<u8>,
        arguments: Vec<u8>,
        attached_deposit: u128,
        prepaid_gas: u64,
    ) -> Result<()> {
        self.receipts.get_mut(receipt_index as usize).unwrap().actions.push(
            VmAction::FunctionCall {
                function_name: String::from_utf8(function_name)
                    // * Unwrap here is fine because this is only used in mocks
                    .expect("method name must be utf8 bytes"),
                args: arguments,
                deposit: attached_deposit,
                gas: prepaid_gas,
            },
        );
        Ok(())
    }

    fn append_action_transfer(&mut self, receipt_index: u64, amount: u128) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(VmAction::Transfer { deposit: amount });
        Ok(())
    }
    
    fn append_action_delete_account(
        &mut self,
        receipt_index: u64,
        beneficiary_id: VmAccountId,
    ) -> Result<()> {
        self.receipts
            .get_mut(receipt_index as usize)
            .ok_or(HostError::InvalidReceiptIndex { receipt_index })?
            .actions
            .push(VmAction::DeleteAccount { beneficiary_id: beneficiary_id.into() });
        Ok(())
    }

    fn get_touched_nodes_count(&self) -> u64 {
        0
    }

    fn reset_touched_nodes_counter(&mut self) {}
}