use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::db::FileDB;
use crate::{ShardTries, Store};
use skw_vm_primitives::account_id::AccountId;
use skw_vm_primitives::contract_runtime::CryptoHash;
use skw_vm_primitives::receipt::{DataReceipt, Receipt, ReceiptEnum};
// use skw_vm_primitives::shard_layout::{ShardUId, ShardVersion};

/// Creates an in-memory database.
pub fn create_test_store() -> Arc<Store> {
    let db = Arc::pin(FileDB::new());
    Arc::new(Store::new(db))
}

/// Creates a Trie using an in-memory database.
pub fn create_tries() -> ShardTries {
    let store = create_test_store();
    ShardTries::new(store)
}

pub fn test_populate_trie(
    tries: &ShardTries,
    root: &CryptoHash,
    changes: Vec<(Vec<u8>, Option<Vec<u8>>)>,
) -> CryptoHash {
    let trie = tries.get_trie();

    // TODO: this sounds wrong??
    // assert_eq!(trie.storage.as_caching_storage().unwrap(), 0);
    
    let trie_changes = trie.update(root, changes.iter().cloned()).unwrap();
    let (store_update, root) = tries.apply_all(&trie_changes).unwrap();
    
    store_update.commit().unwrap();
    let deduped = simplify_changes(&changes);
    for (key, value) in deduped {
        assert_eq!(trie.get(&root, &key), Ok(value));
    }
    root
}

fn gen_accounts(
    rng: &mut impl Rng,
    max_size: usize,
) -> Vec<AccountId> {
    let size = rng.gen_range(0, max_size) + 1;

    std::iter::repeat_with(|| gen_account(rng)).take(size).collect()
}

pub fn gen_account(rng: &mut impl Rng) -> AccountId {
    let key:[u8; 32] = rng.gen();
    let mut key_with_type = Vec::new();
    key_with_type.extend([2u8].iter());
    key_with_type.extend(key);

    AccountId::from_bytes(key_with_type.try_into().unwrap()).unwrap()
}

pub fn gen_unique_accounts(rng: &mut impl Rng, max_size: usize) -> Vec<AccountId> {
    let accounts = gen_accounts(rng, max_size);
    accounts.into_iter().collect::<HashSet<_>>().into_iter().collect()
}

pub fn gen_receipts(rng: &mut impl Rng, max_size: usize) -> Vec<Receipt> {
    let accounts = gen_accounts(rng, max_size);
    accounts
        .iter()
        .map(|account_id| Receipt {
            predecessor_id: account_id.clone(),
            receiver_id: account_id.clone(),
            receipt_id: CryptoHash::default(),
            receipt: ReceiptEnum::Data(DataReceipt { data_id: CryptoHash::default(), data: None }),
        })
        .collect()
}

fn gen_changes_helper(
    rng: &mut impl Rng,
    max_size: usize,
    alphabet: &[u8],
    max_length: u64,
) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
    let mut state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
    let mut result = Vec::new();
    let delete_probability = rng.gen_range(0.1, 0.5);
    let size = rng.gen_range(0, max_size) + 1;
    for _ in 0..size {
        let key_length = rng.gen_range(1, max_length);
        let key: Vec<u8> = (0..key_length).map(|_| *alphabet.choose(rng).unwrap()).collect();

        let delete = rng.gen_range(0.0, 1.0) < delete_probability;
        if delete {
            let mut keys: Vec<_> = state.keys().cloned().collect();
            keys.push(key);
            let key = keys.choose(rng).unwrap().clone();
            state.remove(&key);
            result.push((key.clone(), None));
        } else {
            let value_length = rng.gen_range(1, max_length);
            let value: Vec<u8> =
                (0..value_length).map(|_| *alphabet.choose(rng).unwrap()).collect();
            result.push((key.clone(), Some(value.clone())));
            state.insert(key, value);
        }
    }
    result
}

pub fn gen_changes(rng: &mut impl Rng, max_size: usize) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
    let alphabet = &b"abcdefgh"[0..rng.gen_range(2, 8)];
    let max_length = rng.gen_range(2, 8);
    gen_changes_helper(rng, max_size, alphabet, max_length)
}

pub fn gen_larger_changes(rng: &mut impl Rng, max_size: usize) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
    let alphabet = b"abcdefghijklmnopqrst";
    let max_length = rng.gen_range(10, 20);
    gen_changes_helper(rng, max_size, alphabet, max_length)
}

pub(crate) fn simplify_changes(
    changes: &Vec<(Vec<u8>, Option<Vec<u8>>)>,
) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
    let mut state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
    for (key, value) in changes.iter() {
        if let Some(value) = value {
            state.insert(key.clone(), value.clone());
        } else {
            state.remove(key);
        }
    }
    let mut result: Vec<_> = state.into_iter().map(|(k, v)| (k, Some(v))).collect();
    result.sort();
    result
}
