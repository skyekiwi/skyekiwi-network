use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::ops::Deref;

use borsh::BorshSerialize;
// use skw_vm_primitives::borsh::maybestd::collections::HashMap;
use skw_vm_primitives::trie_key::TrieKey;
use skw_vm_primitives::contract_runtime::{
    CryptoHash, RawStateChange, RawStateChangesWithTrieKey, StateChangeCause, StateRoot,
};

use crate::db::{DBCol, DBOp, DBTransaction};
use crate::trie::trie_storage::{TrieCache, TrieCachingStorage};
use crate::trie::{TrieRefcountChange, POISONED_LOCK_ERR};
use crate::{StorageError, Store, StoreUpdate, Trie, TrieChanges, TrieUpdate};

struct ShardTriesInner {
    store: Arc<Store>,
    /// Cache reserved for client actor to use
    caches: RwLock<TrieCache>,
    /// Cache for readers.
    view_caches: RwLock<TrieCache>,
}

#[derive(Clone)]
pub struct ShardTries(Arc<ShardTriesInner>);

impl ShardTries {
    fn get_new_cache() -> TrieCache {
        TrieCache::new()
    }

    pub fn new(store: Arc<Store>) -> Self {
        ShardTries(Arc::new(ShardTriesInner {
            store,
            caches: RwLock::new(Self::get_new_cache()),
            view_caches: RwLock::new(Self::get_new_cache()),
        }))
    }

    pub fn is_same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn new_trie_update(&self, state_root: CryptoHash) -> TrieUpdate {
        TrieUpdate::new(Rc::new(self.get_trie()), state_root)
    }

    pub fn new_trie_update_view(&self, state_root: CryptoHash) -> TrieUpdate {
        TrieUpdate::new(Rc::new(self.get_view_trie()), state_root)
    }

    fn get_trie_internal(&self, is_view: bool) -> Trie {
        let caches_to_use = if is_view { &self.0.view_caches } else { &self.0.caches };
        let cache = caches_to_use.write().expect(POISONED_LOCK_ERR);
        let store = Box::new(
            TrieCachingStorage::new(self.0.store.clone(), (* cache.deref()).clone())
        );
        Trie::new(store)
    }

    pub fn get_trie(&self) -> Trie {
        self.get_trie_internal(false)
    }

    pub fn get_view_trie(&self) -> Trie {
        self.get_trie_internal(true)
    }

    pub fn get_store(&self) -> Arc<Store> {
        self.0.store.clone()
    }

    pub fn update_cache(&self, transaction: &DBTransaction) -> std::io::Result<()> {
        let mut cache = self.0.caches.write().expect(POISONED_LOCK_ERR);
        
        let mut all_ops: Vec<(CryptoHash, Option<&Vec<u8>>)> = vec![];
        for op in &transaction.ops {
            match op {
                DBOp::UpdateRefcount { col, ref key, ref value } if *col == DBCol::ColState => {    
                    let hash = TrieCachingStorage::get_hash_from_key(key)?;
                    all_ops.push(  (hash, Some(value) ) );
                }
                DBOp::Insert { col, .. } if *col == DBCol::ColState => unreachable!(),
                DBOp::Delete { col, .. } if *col == DBCol::ColState => unreachable!(),
                DBOp::DeleteAll { col } if *col == DBCol::ColState => {
                    // Delete is possible in reset_data_pre_state_sync
                    * cache = Self::get_new_cache();
                }
                _ => {}
            }
        }
        cache.update_cache(all_ops);
        Ok(())
    }

    fn apply_deletions_inner(
        deletions: &Vec<TrieRefcountChange>,
        tries: ShardTries,
        store_update: &mut StoreUpdate,
    ) -> Result<(), StorageError> {
        store_update.tries = Some(tries);
        for TrieRefcountChange { trie_node_or_value_hash, trie_node_or_value, rc } in
            deletions.iter()
        {
            store_update.update_refcount(
                DBCol::ColState,
                trie_node_or_value_hash,
                trie_node_or_value,
                -(*rc as i64),
            );
        }
        Ok(())
    }

    fn apply_insertions_inner(
        insertions: &Vec<TrieRefcountChange>,
        tries: ShardTries,
        store_update: &mut StoreUpdate,
    ) -> Result<(), StorageError> {
        store_update.tries = Some(tries);
        for TrieRefcountChange { trie_node_or_value_hash, trie_node_or_value, rc } in
            insertions.iter()
        {
            store_update.update_refcount(
                DBCol::ColState,
                trie_node_or_value_hash,
                trie_node_or_value,
                *rc as i64,
            );
        }
        Ok(())
    }

    fn apply_all_inner(
        trie_changes: &TrieChanges,
        tries: ShardTries,
        apply_deletions: bool,
    ) -> Result<(StoreUpdate, StateRoot), StorageError> {
        let mut store_update = StoreUpdate::new_with_tries(tries.clone());
        ShardTries::apply_insertions_inner(
            &trie_changes.insertions,
            tries.clone(),
            &mut store_update,
        )?;
        if apply_deletions {
            ShardTries::apply_deletions_inner(
                &trie_changes.deletions,
                tries,
                &mut store_update,
            )?;
        }
        Ok((store_update, trie_changes.new_root))
    }

    pub fn apply_insertions(
        &self,
        trie_changes: &TrieChanges,
        store_update: &mut StoreUpdate,
    ) -> Result<(), StorageError> {
        ShardTries::apply_insertions_inner(
            &trie_changes.insertions,
            self.clone(),
            store_update,
        )
    }

    pub fn apply_deletions(
        &self,
        trie_changes: &TrieChanges,
        store_update: &mut StoreUpdate,
    ) -> Result<(), StorageError> {
        ShardTries::apply_deletions_inner(
            &trie_changes.deletions,
            self.clone(),
            store_update,
        )
    }

    pub fn revert_insertions(
        &self,
        trie_changes: &TrieChanges,
        store_update: &mut StoreUpdate,
    ) -> Result<(), StorageError> {
        ShardTries::apply_deletions_inner(
            &trie_changes.insertions,
            self.clone(),
            store_update,
        )
    }

    pub fn apply_all(
        &self,
        trie_changes: &TrieChanges,
    ) -> Result<(StoreUpdate, StateRoot), StorageError> {
        ShardTries::apply_all_inner(trie_changes, self.clone(), true)
    }

    // apply_all with less memory overhead
    pub fn apply_genesis(
        &self,
        trie_changes: TrieChanges,
    ) -> (StoreUpdate, StateRoot) {
        assert_eq!(trie_changes.old_root, CryptoHash::default());
        assert!(trie_changes.deletions.is_empty());
        // Not new_with_tries on purpose
        let mut store_update = StoreUpdate::new(self.get_store().storage.clone());
        for TrieRefcountChange { trie_node_or_value_hash, trie_node_or_value, rc } in
            trie_changes.insertions.into_iter()
        {
            store_update.update_refcount(
                DBCol::ColState,
                &trie_node_or_value_hash,
                &trie_node_or_value,
                rc as i64,
            );
        }
        (store_update, trie_changes.new_root)
    }
}

pub struct WrappedTrieChanges {
    tries: ShardTries,
    trie_changes: TrieChanges,
    state_changes: Vec<RawStateChangesWithTrieKey>,
    block_hash: CryptoHash,
}

impl WrappedTrieChanges {
    pub fn new(
        tries: ShardTries,
        trie_changes: TrieChanges,
        state_changes: Vec<RawStateChangesWithTrieKey>,
        block_hash: CryptoHash,
    ) -> Self {
        WrappedTrieChanges { tries, trie_changes, state_changes, block_hash }
    }

    pub fn state_changes(&self) -> &[RawStateChangesWithTrieKey] {
        &self.state_changes
    }

    pub fn insertions_into(&self, store_update: &mut StoreUpdate) -> Result<(), StorageError> {
        self.tries.apply_insertions(&self.trie_changes, store_update)
    }

    /// Save state changes into Store.
    ///
    /// NOTE: the changes are drained from `self`.
    pub fn state_changes_into(&mut self, store_update: &mut StoreUpdate) {
        for change_with_trie_key in self.state_changes.drain(..) {
            assert!(
                !change_with_trie_key.changes.iter().any(|RawStateChange { cause, .. }| matches!(
                    cause,
                    StateChangeCause::NotWritableToDisk
                )),
                "NotWritableToDisk changes must never be finalized."
            );

            // TODO: remove!
            assert!(
                !change_with_trie_key.changes.iter().any(|RawStateChange { cause, .. }| matches!(
                    cause,
                    StateChangeCause::Resharding
                )),
                "Resharding changes must never be finalized."
            );

            // Filtering trie keys for user facing RPC reporting.
            // NOTE: If the trie key is not one of the account specific, it may cause key conflict
            // when the node tracks multiple shards. See #2563.
            match &change_with_trie_key.trie_key {
                TrieKey::Account { .. }
                | TrieKey::ContractCode { .. }
                | TrieKey::ContractData { .. } => {}
                _ => continue,
            };
            let storage_key = KeyForStateChanges::new_from_trie_key(
                &self.block_hash,
                &change_with_trie_key.trie_key,
            );
            store_update.set(
                DBCol::ColStateChanges,
                storage_key.as_ref(),
                &change_with_trie_key.try_to_vec().expect("Borsh serialize cannot fail"),
            );
        }
    }

    pub fn wrapped_into(
        &mut self,
        store_update: &mut StoreUpdate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.insertions_into(store_update)?;
        self.state_changes_into(store_update);
        store_update.set_ser(
            DBCol::ColTrieChanges,
            &self.block_hash,
            &self.trie_changes,
        )?;
        Ok(())
    }
}

#[derive(derive_more::AsRef, derive_more::Into)]
pub struct KeyForStateChanges(Vec<u8>);

impl KeyForStateChanges {
    fn estimate_prefix_len() -> usize {
        std::mem::size_of::<CryptoHash>()
    }

    fn get_prefix_with_capacity(block_hash: &CryptoHash, reserve_capacity: usize) -> Self {
        let mut key_prefix = Vec::with_capacity(Self::estimate_prefix_len() + reserve_capacity);
        key_prefix.extend(block_hash.as_ref());
        debug_assert_eq!(key_prefix.len(), Self::estimate_prefix_len());
        Self(key_prefix)
    }

    pub fn get_prefix(block_hash: &CryptoHash) -> Self {
        Self::get_prefix_with_capacity(block_hash, 0)
    }

    pub fn new(block_hash: &CryptoHash, raw_key: &[u8]) -> Self {
        let mut key = Self::get_prefix_with_capacity(block_hash, raw_key.len());
        key.0.extend(raw_key);
        key
    }

    pub fn new_from_trie_key(block_hash: &CryptoHash, trie_key: &TrieKey) -> Self {
        let mut key = Self::get_prefix_with_capacity(block_hash, trie_key.len());
        key.0.extend(trie_key.to_vec());
        key
    }

    pub fn find_iter<'a: 'b, 'b>(
        &'a self,
        store: &'b Store,
    ) -> impl Iterator<Item = Result<RawStateChangesWithTrieKey, std::io::Error>> + 'b {
        let prefix_len = Self::estimate_prefix_len();
        debug_assert!(self.0.len() >= prefix_len);
        store.iter_prefix_ser::<RawStateChangesWithTrieKey>(DBCol::ColStateChanges, &self.0).map(
            move |change| {
                // Split off the irrelevant part of the key, so only the original trie_key is left.
                let (key, state_changes) = change?;
                debug_assert!(key.starts_with(&self.0));
                Ok(state_changes)
            },
        )
    }

    pub fn find_exact_iter<'a: 'b, 'b>(
        &'a self,
        store: &'b Store,
    ) -> impl Iterator<Item = Result<RawStateChangesWithTrieKey, std::io::Error>> + 'b {
        let prefix_len = Self::estimate_prefix_len();
        let trie_key_len = self.0.len() - prefix_len;
        self.find_iter(store).filter_map(move |change| {
            let state_changes = match change {
                Ok(change) => change,
                error => {
                    return Some(error);
                }
            };
            if state_changes.trie_key.len() != trie_key_len {
                None
            } else {
                debug_assert_eq!(&state_changes.trie_key.to_vec()[..], &self.0[prefix_len..]);
                Some(Ok(state_changes))
            }
        })
    }
}
