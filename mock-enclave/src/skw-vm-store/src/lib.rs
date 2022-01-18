use hashbrown::HashMap;
use skw_vm_primitives::db_key::DBKey;
use skw_vm_primitives::{
    contract_runtime::ContractCode,
    account_id: AccountId,
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Store(HashMap<DBKey, Vec<u8>>);

impl Store {
    pub fn get(&self, key: &DBKey) -> Option<Vec<u8>> {
        self.0.get(key)
    }

    pub fn remove(&mut self, key: &DBKey) ->  bool {
        self.0.remove(key).is_some()
    }

    /// will replace content if occupied
    pub fn force_set(&mut self, key: &DBKey, content: Vec<u8>) -> bool {
        self.0.insert(key, content);
        true
    }

    /// may fail when the key is occupied
    pub fn try_set(&mut self, key: &DBKey, content: Vec<u8>) -> bool {
        self.0.try_insert(key, content).is_ok()
    }
}

pub fn get_code(store: &Store, account_id: AccountId) -> Option<ContractCode> {
    match store.get(&DBKey::ContractCode {account_id: account_id})) {
        Some(code) => ContractCode::new(code),
        None => None,
    }
}

// code
pub fn try_set_code(store: &mut Store, account_id: AccountId, code: &ContractCode) -> bool {
    store.try_set(&DBKey::ContractCode {account_id: account_id}, code.code)
}

pub fn force_set_code(store: &mut Store, account_id: AccountId, code: &ContractCode) -> bool {
    store.force_set(&DBKey::ContractCode {account_id: account_id}, code.code)
}

pub fn get_code(store: &mut Store, account_id: AccountId) -> bool {
    store.remove(&DBKey::ContractCode {account_id: account_id})
}

// account
pub fn try_set_code(store: &mut Store, account_id: AccountId, code: &ContractCode) -> bool {
    store.try_set(&DBKey::ContractCode {account_id: account_id}, code.code)
}

pub fn force_set_code(store: &mut Store, account_id: AccountId, code: &ContractCode) -> bool {
    store.force_set(&DBKey::ContractCode {account_id: account_id}, code.code)
}

pub fn get_code(store: &mut Store, account_id: AccountId) -> bool {
    store.remove(&DBKey::ContractCode {account_id: account_id})
}
