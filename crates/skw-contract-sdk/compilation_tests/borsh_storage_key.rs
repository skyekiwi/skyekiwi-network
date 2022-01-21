//! Testing BorshStorageKey macro.

use borsh::{BorshDeserialize, BorshSerialize};
use skw_contract_sdk::collections::LookupMap;
use skw_contract_sdk::{skw_bindgen, BorshStorageKey};

#[derive(BorshStorageKey, BorshSerialize)]
struct StorageKeyStruct {
    key: String,
}

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKeyEnum {
    Accounts,
    SubAccounts { account_id: String },
}

#[skw_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    map1: LookupMap<u64, u64>,
    map2: LookupMap<String, String>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            map1: LookupMap::new(StorageKeyStruct { key: "bla".to_string() }),
            map2: LookupMap::new(StorageKeyEnum::Accounts),
        }
    }
}

#[skw_bindgen]
impl Contract {}

fn main() {}
