use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::{fmt, io};

use borsh::{BorshDeserialize, BorshSerialize};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lru::LruCache;

mod refcount;
pub use db::DBCol::{self, *};
pub use db::{
    CHUNK_TAIL_KEY, FINAL_HEAD_KEY, FORK_TAIL_KEY, HEADER_HEAD_KEY, HEAD_KEY,
    LARGEST_TARGET_HEIGHT_KEY, LATEST_KNOWN_KEY, NUM_COLS, SHOULD_COL_GC, SKIP_COL_GC, TAIL_KEY,
};

use skw_vm_primitives::crypto::PublicKey;
use skw_vm_primitives::account::{Account, AccessKey};
pub use skw_vm_primitives::errors::StorageError;
use skw_vm_primitives::contract_runtime::{CryptoHash, ContractCode, AccountId, StateRoot};
use skw_vm_primitives::receipt::{DelayedReceiptIndices, Receipt, ReceivedData};
use skw_vm_primitives::serialize::to_base;
use skw_vm_primitives::trie_key::{trie_key_parsers, TrieKey};
use skw_myers_diff::{diff, diff_ops_to_bytes, bytes_to_diff_ops};
pub use crate::refcount::decode_value_with_rc;
use crate::refcount::encode_value_with_rc;
use crate::db::{
    DBOp, DBTransaction, Database, FileDB, GENESIS_JSON_HASH_KEY, GENESIS_STATE_ROOTS_KEY,
};
pub use crate::trie::{
    iterator::TrieIterator, update::TrieUpdate, update::TrieUpdateIterator,
    update::TrieUpdateValuePtr, ApplyStatePartResult, KeyForStateChanges, PartialStorage,
    ShardTries, Trie, TrieChanges, WrappedTrieChanges,
};

pub mod db;
pub mod test_utils;
mod trie;

#[derive(Clone)]
pub struct Store {
    storage: Pin<Arc<dyn Database>>,
}

impl Store {
    pub fn new(storage: Pin<Arc<dyn Database>>) -> Store {
        Store { storage }
    }

    pub fn get(&self, column: DBCol, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(column, key)
    }

    pub fn get_ser<T: BorshDeserialize>(
        &self,
        column: DBCol,
        key: &[u8],
    ) -> Result<Option<T>, io::Error> {
        match self.storage.get(column, key) {
            Some(bytes) => match T::try_from_slice(bytes.as_ref()) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    pub fn exists(&self, column: DBCol, key: &[u8]) -> bool {
        self.storage.get(column, key).is_some()
    }

    pub fn store_update(&self) -> StoreUpdate {
        StoreUpdate::new(self.storage.clone())
    }

    pub fn iter<'a>(
        &'a self,
        column: DBCol,
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        self.storage.iter(column)
    }

    pub fn iter_without_rc_logic<'a>(
        &'a self,
        column: DBCol,
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        self.storage.iter_without_rc_logic(column)
    }

    pub fn iter_prefix<'a>(
        &'a self,
        column: DBCol,
        key_prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        self.storage.iter_prefix(column, key_prefix)
    }

    pub fn iter_prefix_ser<'a, T: BorshDeserialize>(
        &'a self,
        column: DBCol,
        key_prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = Result<(Vec<u8>, T), io::Error>> + 'a> {
        Box::new(
            self.storage
                .iter_prefix(column, key_prefix)
                .map(|(key, value)| Ok((key.to_vec(), T::try_from_slice(value.as_ref())?))),
        )
    }

    pub fn generate_patch_on_air(&self, origin_filename_prefix: &str) -> Result<Vec<u8>, std::io::Error> {
        let origin_file = File::open(
            Path::new(&format!("{}__state_dump__{:?}", origin_filename_prefix, DBCol::ColState))
        )?;
        let mut origin = Vec::new();
        BufReader::new(origin_file).read(&mut origin[..])?;

        let dest = self.save_to_buf(DBCol::ColState)?;
        let diff = diff(&origin[..], &dest[..]);
        Ok(diff_ops_to_bytes(diff))
    }

    pub fn generate_patch(origin_filename_prefix: &str, dest_file_name_prefix: &str) -> Result<Vec<u8>, std::io::Error> {
        let origin_file = File::open(
            Path::new(&format!("{}__state_dump__{:?}", origin_filename_prefix, DBCol::ColState))
        )?;
        let dest_file = File::open(
            Path::new(&format!("{}__state_dump__{:?}", dest_file_name_prefix, DBCol::ColState))
        )?;

        let mut origin = Vec::new();
        let mut dest = Vec::new();

        BufReader::new(origin_file).read(&mut origin[..])?;
        BufReader::new(dest_file).read(&mut dest[..])?;

        let diff = diff(&origin[..], &dest[..]);
        Ok(diff_ops_to_bytes(diff))
    }

    pub fn read_from_patch(&self, origin_filename_prefix: &str, patch: &[u8]) -> Result<(), std::io::Error> {
        let origin_file = File::open(
            Path::new(&format!("{}__state_dump__{:?}", origin_filename_prefix, DBCol::ColState))
        )?;

        let diff = bytes_to_diff_ops(patch);
        
        let mut origin = Vec::new();
        BufReader::new(origin_file).read(&mut origin[..])?;
        let dest = skw_myers_diff::patch(diff, &origin[..]);

        self.load_from_buf(DBCol::ColState, &dest)?;
        Ok(())
    }

    pub fn save_state_to_file(&self, filename_prefix: &str) -> Result<(), std::io::Error> {
        self.save_to_file(DBCol::ColState,
            Path::new(&format!("{}__state_dump__{:?}", filename_prefix, DBCol::ColState))
        )
    }

    pub fn load_state_from_file(&self, filename_prefix: &str) -> Result<(), std::io::Error> {
        self.load_from_file(DBCol::ColState,
            Path::new(&format!("{}__state_dump__{:?}", filename_prefix, DBCol::ColState))
        )
    }

    pub fn save_to_file(&self, column: DBCol, filename: &Path) -> Result<(), std::io::Error> {
        let mut file = File::create(filename)?;
        for (key, value) in self.storage.iter_without_rc_logic(column) {
            file.write_u32::<LittleEndian>(key.len() as u32)?;
            file.write_all(&key)?;
            file.write_u32::<LittleEndian>(value.len() as u32)?;
            file.write_all(&value)?;
        }
        Ok(())
    }

    pub fn save_to_buf(&self, column: DBCol) -> Result<Vec<u8>, std::io::Error> {
        let mut res = Vec::new();
        for (key, value) in self.storage.iter_without_rc_logic(column) {
            res.write_u32::<LittleEndian>(key.len() as u32)?;
            res.write_all(&key)?;
            res.write_u32::<LittleEndian>(value.len() as u32)?;
            res.write_all(&value)?;
        }
        Ok(res)
    }

    pub fn print_db(&self) -> Result<(), std::io::Error> {
        for (key, value) in self.storage.iter_without_rc_logic(DBCol::ColState) {
            println!("Key {:?}", &key);
            println!("Value {:?}", &value);
        }
        Ok(())
    }

    pub fn load_from_buf(&self, column: DBCol, mut buf: &[u8]) -> Result<(), std::io::Error> {
        let mut transaction = self.storage.transaction();
        let mut key = Vec::new();
        let mut value = Vec::new();
        loop {
            let key_len = match buf.read_u32::<LittleEndian>() {
                Ok(key_len) => key_len as usize,
                Err(ref err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err),
            };
            key.resize(key_len, 0);
            buf.read_exact(&mut key)?;

            let value_len = buf.read_u32::<LittleEndian>()? as usize;
            value.resize(value_len, 0);
            buf.read_exact(&mut value)?;

            transaction.put(column, &key, &value);
        }
        self.storage.write(transaction);
        Ok(())
    }

    pub fn load_from_file(&self, column: DBCol, filename: &Path) -> Result<(), std::io::Error> {
        let file = File::open(filename)?;
        let mut file = BufReader::new(file);
        let mut transaction = self.storage.transaction();
        let mut key = Vec::new();
        let mut value = Vec::new();
        loop {
            let key_len = match file.read_u32::<LittleEndian>() {
                Ok(key_len) => key_len as usize,
                Err(ref err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err),
            };
            key.resize(key_len, 0);
            file.read_exact(&mut key)?;

            let value_len = file.read_u32::<LittleEndian>()? as usize;
            value.resize(value_len, 0);
            file.read_exact(&mut value)?;

            transaction.put(column, &key, &value);
        }
        self.storage.write(transaction);
        Ok(())
    }
}

/// Keeps track of current changes to the database and can commit all of them to the database.
pub struct StoreUpdate {
    storage: Pin<Arc<dyn Database>>,
    transaction: DBTransaction,
    /// Optionally has reference to the trie to clear cache on the commit.
    tries: Option<ShardTries>,
}

impl StoreUpdate {
    pub fn new(storage: Pin<Arc<dyn Database>>) -> Self {
        let transaction = storage.transaction();
        StoreUpdate { storage, transaction, tries: None }
    }

    pub fn new_with_tries(tries: ShardTries) -> Self {
        let storage = tries.get_store().storage.clone();
        let transaction = storage.transaction();
        StoreUpdate { storage, transaction, tries: Some(tries) }
    }

    pub fn update_refcount(&mut self, column: DBCol, key: &[u8], value: &[u8], rc_delta: i64) {
        debug_assert!(column.is_rc());
        let value = encode_value_with_rc(value, rc_delta);
        self.transaction.update_refcount(column, key, value)
    }

    pub fn set(&mut self, column: DBCol, key: &[u8], value: &[u8]) {
        self.transaction.put(column, key, value)
    }

    pub fn set_ser<T: BorshSerialize>(
        &mut self,
        column: DBCol,
        key: &[u8],
        value: &T,
    ) -> Result<(), io::Error> {
        debug_assert!(!column.is_rc());
        let data = value.try_to_vec()?;
        self.set(column, key, &data);
        Ok(())
    }

    pub fn delete(&mut self, column: DBCol, key: &[u8]) {
        self.transaction.delete(column, key);
    }

    pub fn delete_all(&mut self, column: DBCol) {
        self.transaction.delete_all(column);
    }

    /// Merge another store update into this one.
    pub fn merge(&mut self, other: StoreUpdate) {
        if let Some(tries) = other.tries {
            if self.tries.is_none() {
                self.tries = Some(tries);
            } else {
                debug_assert!(self.tries.as_ref().unwrap().is_same(&tries));
            }
        }

        self.merge_transaction(other.transaction);
    }

    /// Merge DB Transaction.
    pub fn merge_transaction(&mut self, transaction: DBTransaction) {
        for op in transaction.ops {
            match op {
                DBOp::Insert { col, key, value } => self.transaction.put(col, &key, &value),
                DBOp::Delete { col, key } => self.transaction.delete(col, &key),
                DBOp::UpdateRefcount { col, key, value } => {
                    self.transaction.update_refcount(col, &key, &value)
                }
                DBOp::DeleteAll { col } => self.transaction.delete_all(col),
            }
        }
    }

    pub fn commit(self) -> Result<(), io::Error> {
        debug_assert!(
            {
                let non_refcount_keys = self
                    .transaction
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        DBOp::Insert { col, key, .. } => Some((*col as u8, key)),
                        DBOp::Delete { col, key } => Some((*col as u8, key)),
                        DBOp::UpdateRefcount { .. } => None,
                        DBOp::DeleteAll { .. } => None,
                    })
                    .collect::<Vec<_>>();
                non_refcount_keys.len()
                    == non_refcount_keys.iter().collect::<std::collections::HashSet<_>>().len()
            },
            "Transaction overwrites itself: {:?}",
            self
        );
        if let Some(tries) = self.tries {
            assert_eq!(
                tries.get_store().storage.deref() as *const _,
                self.storage.deref() as *const _
            );
            tries.update_cache(&self.transaction)?;
        }
        self.storage.write(self.transaction);
        Ok(())
    }
}

impl fmt::Debug for StoreUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Store Update {{")?;
        for op in self.transaction.ops.iter() {
            match op {
                DBOp::Insert { col, key, .. } => writeln!(f, "  + {:?} {}", col, to_base(key))?,
                DBOp::UpdateRefcount { col, key, .. } => {
                    writeln!(f, "  +- {:?} {}", col, to_base(key))?
                }
                DBOp::Delete { col, key } => writeln!(f, "  - {:?} {}", col, to_base(key))?,
                DBOp::DeleteAll { col } => writeln!(f, "  delete all {:?}", col)?,
            }
        }
        writeln!(f, "}}")
    }
}

pub fn read_with_cache<'a, T: BorshDeserialize + 'a>(
    storage: &Store,
    col: DBCol,
    cache: &'a mut LruCache<Vec<u8>, T>,
    key: &[u8],
) -> io::Result<Option<&'a T>> {
    let key_vec = key.to_vec();
    if cache.get(&key_vec).is_some() {
        return Ok(Some(cache.get(&key_vec).unwrap()));
    }
    if let Some(result) = storage.get_ser(col, key)? {
        cache.put(key.to_vec(), result);
        return Ok(cache.get(&key_vec));
    }
    Ok(None)
}

pub fn create_store() -> Arc<Store> {
    let db = Arc::pin(FileDB::new());
    Arc::new(Store::new(db))
}

/// Reads an object from Trie.
/// # Errors
/// see StorageError
pub fn get<T: BorshDeserialize>(
    state_update: &TrieUpdate,
    key: &TrieKey,
) -> Result<Option<T>, StorageError> {
    state_update.get(key).and_then(|opt| {
        opt.map_or_else(
            || Ok(None),
            |data| {
                T::try_from_slice(&data)
                    .map_err(|_| {
                        StorageError::StorageInconsistentState("Failed to deserialize".to_string())
                    })
                    .map(Some)
            },
        )
    })
}

/// Writes an object into Trie.
pub fn set<T: BorshSerialize>(state_update: &mut TrieUpdate, key: TrieKey, value: &T) {
    let data = value.try_to_vec().expect("Borsh serializer is not expected to ever fail");
    state_update.set(key, data);
}

pub fn set_account(state_update: &mut TrieUpdate, account_id: AccountId, account: &Account) {
    set(state_update, TrieKey::Account { account_id }, account)
}

pub fn get_account(
    state_update: &TrieUpdate,
    account_id: &AccountId,
) -> Result<Option<Account>, StorageError> {
    get(state_update, &TrieKey::Account { account_id: account_id.clone() })
}

pub fn set_received_data(
    state_update: &mut TrieUpdate,
    receiver_id: AccountId,
    data_id: CryptoHash,
    data: &ReceivedData,
) {
    set(state_update, TrieKey::ReceivedData { receiver_id, data_id }, data);
}

pub fn get_received_data(
    state_update: &TrieUpdate,
    receiver_id: &AccountId,
    data_id: CryptoHash,
) -> Result<Option<ReceivedData>, StorageError> {
    get(state_update, &TrieKey::ReceivedData { receiver_id: receiver_id.clone(), data_id })
}

pub fn set_postponed_receipt(state_update: &mut TrieUpdate, receipt: &Receipt) {
    let key = TrieKey::PostponedReceipt {
        receiver_id: receipt.receiver_id.clone(),
        receipt_id: receipt.receipt_id,
    };
    set(state_update, key, receipt);
}

pub fn remove_postponed_receipt(
    state_update: &mut TrieUpdate,
    receiver_id: &AccountId,
    receipt_id: CryptoHash,
) {
    state_update.remove(TrieKey::PostponedReceipt { receiver_id: receiver_id.clone(), receipt_id });
}

pub fn get_postponed_receipt(
    state_update: &TrieUpdate,
    receiver_id: &AccountId,
    receipt_id: CryptoHash,
) -> Result<Option<Receipt>, StorageError> {
    get(state_update, &TrieKey::PostponedReceipt { receiver_id: receiver_id.clone(), receipt_id })
}

pub fn get_delayed_receipt_indices(
    state_update: &TrieUpdate,
) -> Result<DelayedReceiptIndices, StorageError> {
    Ok(get(state_update, &TrieKey::DelayedReceiptIndices)?.unwrap_or_default())
}

pub fn set_access_key(
    state_update: &mut TrieUpdate,
    account_id: AccountId,
    public_key: PublicKey,
    access_key: &AccessKey,
) {
    set(state_update, TrieKey::AccessKey { account_id, public_key }, access_key);
}

pub fn remove_access_key(
    state_update: &mut TrieUpdate,
    account_id: AccountId,
    public_key: PublicKey,
) {
    state_update.remove(TrieKey::AccessKey { account_id, public_key });
}

pub fn get_access_key(
    state_update: &TrieUpdate,
    account_id: &AccountId,
    public_key: &PublicKey,
) -> Result<Option<AccessKey>, StorageError> {
    get(
        state_update,
        &TrieKey::AccessKey { account_id: account_id.clone(), public_key: public_key.clone() },
    )
}

pub fn get_access_key_raw(
    state_update: &TrieUpdate,
    raw_key: &[u8],
) -> Result<Option<AccessKey>, StorageError> {
    get(
        state_update,
        &trie_key_parsers::parse_trie_key_access_key_from_raw_key(raw_key)
            .expect("access key in the state should be correct"),
    )
}

pub fn set_code(state_update: &mut TrieUpdate, account_id: AccountId, code: &ContractCode) {
    state_update.set(TrieKey::ContractCode { account_id }, code.code.to_vec());
}

pub fn get_code(
    state_update: &TrieUpdate,
    account_id: &AccountId,
    _code_hash: Option<CryptoHash>,
) -> Result<Option<ContractCode>, StorageError> {
    state_update
        .get(&TrieKey::ContractCode { account_id: account_id.clone() })
        .map(|opt| opt.map(|code| ContractCode::new(&code)))
}

/// Removes account, code and all access keys associated to it.
pub fn remove_account(
    state_update: &mut TrieUpdate,
    account_id: &AccountId,
) -> Result<(), StorageError> {
    state_update.remove(TrieKey::Account { account_id: account_id.clone() });
    state_update.remove(TrieKey::ContractCode { account_id: account_id.clone() });

    // Removing access keys
    let public_keys = state_update
        .iter(&trie_key_parsers::get_raw_prefix_for_access_keys(account_id))?
        .map(|raw_key| {
            trie_key_parsers::parse_public_key_from_access_key_key(&raw_key?, account_id).map_err(
                |_e| {
                    StorageError::StorageInconsistentState(
                        "Can't parse public key from raw key for AccessKey".to_string(),
                    )
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    for public_key in public_keys {
        state_update.remove(TrieKey::AccessKey { account_id: account_id.clone(), public_key });
    }

    // Removing contract data
    let data_keys = state_update
        .iter(&trie_key_parsers::get_raw_prefix_for_contract_data(account_id, &[]))?
        .map(|raw_key| {
            trie_key_parsers::parse_data_key_from_contract_data_key(&raw_key?, account_id)
                .map_err(|_e| {
                    StorageError::StorageInconsistentState(
                        "Can't parse data key from raw key for ContractData".to_string(),
                    )
                })
                .map(Vec::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for key in data_keys {
        state_update.remove(TrieKey::ContractData { account_id: account_id.clone(), key });
    }
    Ok(())
}

pub fn get_genesis_state_roots(store: &Store) -> Result<Option<Vec<StateRoot>>, std::io::Error> {
    store.get_ser::<Vec<StateRoot>>(DBCol::ColBlockMisc, GENESIS_STATE_ROOTS_KEY)
}

pub fn get_genesis_hash(store: &Store) -> Result<Option<CryptoHash>, std::io::Error> {
    store.get_ser::<CryptoHash>(DBCol::ColBlockMisc, GENESIS_JSON_HASH_KEY)
}

pub fn set_genesis_hash(store_update: &mut StoreUpdate, genesis_hash: &CryptoHash) {
    store_update
        .set_ser::<CryptoHash>(DBCol::ColBlockMisc, GENESIS_JSON_HASH_KEY, genesis_hash)
        .expect("Borsh cannot fail");
}

pub fn set_genesis_state_roots(store_update: &mut StoreUpdate, genesis_roots: &Vec<StateRoot>) {
    store_update
        .set_ser::<Vec<StateRoot>>(DBCol::ColBlockMisc, GENESIS_STATE_ROOTS_KEY, genesis_roots)
        .expect("Borsh cannot fail");
}


#[cfg(test)]
mod tests {
    use crate::db::DBCol::ColState;
    use crate::{create_store};

    #[test]
    fn test_write_read_from_file() {

        {
            let store = create_store();
            assert_eq!(store.get(ColState, &[1]), None);
            {
                let mut store_update = store.store_update();
                store_update.update_refcount(ColState, &[1], &[1], 1);
                store_update.update_refcount(ColState, &[2], &[2], 1);
                store_update.update_refcount(ColState, &[3], &[3], 1);
                store_update.commit().unwrap();
            }
            assert_eq!(store.get(ColState, &[1]), Some(vec![1]));

            store.save_state_to_file("./mock/test").unwrap();
        }

        {
            let store = create_store();
            store.load_state_from_file("./mock/test").unwrap();
            assert_eq!(store.get(ColState, &[1]), Some(vec![1]));
        }
    }

    // #[test]
    // fn test_read_empty_file() {
    //     let store = create_store();
    //     store.load_state_from_file("./mock/empty").unwrap();
    //     store.print_db();
    // }

    #[test]
    fn test_file_patching() {
        {
            let store = create_store();
            assert_eq!(store.get(ColState, &[1]), None);
            {
                let mut store_update = store.store_update();
                store_update.update_refcount(ColState, &[1], &[1], 1);
                store_update.update_refcount(ColState, &[2], &[2], 1);
                store_update.update_refcount(ColState, &[3], &[3], 1);
                store_update.commit().unwrap();
            }
            assert_eq!(store.get(ColState, &[1]), Some(vec![1]));

            store.save_state_to_file("./mock/test").unwrap();
        }

        let patch_bytes = || {
            let store = create_store();
            store.load_state_from_file("./mock/test").unwrap();
            {
                let mut store_update = store.store_update();
                store_update.update_refcount(ColState, &[4], &[4], 1);
                store_update.commit().unwrap();
            }
            
            store.generate_patch_on_air("./mock/test").unwrap()
        };

        {
            let store = create_store();
            store.read_from_patch("./mock/test", &patch_bytes()).unwrap();
            assert_eq!(store.get(ColState, &[4]), Some(vec![4]));
        }
    }
}
