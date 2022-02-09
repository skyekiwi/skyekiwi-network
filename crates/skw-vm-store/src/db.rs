use std::sync::RwLock;

use borsh::{BorshDeserialize, BorshSerialize};
use strum::EnumIter;

use crate::refcount::merge_refcounted_records;

/// This enum holds the information about the columns that we use within the RocksDB storage.
/// You can think about our storage as 2-dimensional table (with key and column as indexes/coordinates).
// TODO(mm-near): add info about the RC in the columns.
#[derive(PartialEq, Debug, Copy, Clone, EnumIter, BorshDeserialize, BorshSerialize, Hash, Eq)]
pub enum DBCol {
    /// Column to indicate which version of database this is.
    /// - *Rows*: single row [VERSION_KEY]
    /// - *Content type*: The version of the database (u32), serialized as JSON.
    ColDbVersion = 0,

    /// Column that store Misc cells.
    /// - *Rows*: multiple, for example "GENESIS_JSON_HASH", "HEAD_KEY", [LATEST_KNOWN_KEY] etc.
    /// - *Content type*: cell specific.
    ColBlockMisc = 1,
    /// Column that stores Block content.
    /// - *Rows*: block hash (CryptHash)
    /// - *Content type*: [near_primitives::block::Block]
    ColBlock = 2,
    /// Column that stores Block headers.
    /// - *Rows*: block hash (CryptoHash)
    /// - *Content type*: [near_primitives::block_header::BlockHeader]
    ColBlockHeader = 3,
    /// Column that stores mapping from block height to block hash.
    /// - *Rows*: height (u64)
    /// - *Content type*: block hash (CryptoHash)
    ColBlockHeight = 4,
    /// Column that stores the Trie state.
    /// - *Rows*: trie_node_or_value_hash (CryptoHash)
    /// - *Content type*: Serializd RawTrieNodeWithSize or value ()
    ColState = 5,
    /// Mapping from BlockChunk to ChunkExtra
    /// - *Rows*: BlockChunk (block_hash, shard_uid)
    /// - *Content type*: [near_primitives::types::ChunkExtra]
    ColChunkExtra = 6,
    /// Mapping from transaction outcome id (CryptoHash) to list of outcome ids with proofs.
    /// - *Rows*: outcome id (CryptoHash)
    /// - *Content type*: Vec of [near_primitives::transactions::ExecutionOutcomeWithIdAndProof]
    ColTransactionResult = 7,
    /// Mapping from Block + Shard to list of outgoing receipts.
    /// - *Rows*: block + shard
    /// - *Content type*: Vec of [near_primitives::receipt::Receipt]
    ColOutgoingReceipts = 8,
    /// Mapping from Block + Shard to list of incoming receipt proofs.
    /// Each proof might prove multiple receipts.
    /// - *Rows*: (block, shard)
    /// - *Content type*: Vec of [near_primitives::sharding::ReceiptProof]
    ColIncomingReceipts = 9,
    /// Info about the peers that we are connected to. Mapping from peer_id to KnownPeerState.
    /// - *Rows*: peer_id (PublicKey)
    /// - *Content type*: [network_primitives::types::KnownPeerState]
    ColPeers = 10,
    /// Mapping from EpochId to EpochInfo
    /// - *Rows*: EpochId (CryptoHash)
    /// - *Content type*: [near_primitives::epoch_manager::EpochInfo]
    ColEpochInfo = 11,
    /// Mapping from BlockHash to BlockInfo
    /// - *Rows*: BlockHash (CryptoHash)
    /// - *Content type*: [near_primitives::epoch_manager::BlockInfo]
    ColBlockInfo = 12,
    /// Mapping from ChunkHash to ShardChunk.
    /// - *Rows*: ChunkHash (CryptoHash)
    /// - *Content type*: [near_primitives::sharding::ShardChunk]
    ColChunks = 13,
    /// Storage for  PartialEncodedChunk.
    /// - *Rows*: ChunkHash (CryptoHash)
    /// - *Content type*: [near_primitives::sharding::PartialEncodedChunkV1]
    ColPartialChunks = 14,
    /// Blocks for which chunks need to be applied after the state is downloaded for a particular epoch
    /// TODO: describe what is exactly inside the rows/cells.
    ColBlocksToCatchup = 15,
    /// Blocks for which the state is being downloaded
    ColStateDlInfos = 16,
    ColChallengedBlocks = 17,
    ColStateHeaders = 18,
    ColInvalidChunks = 19,
    ColBlockExtra = 20,
    /// Store hash of a block per each height, to detect double signs.
    ColBlockPerHeight = 21,
    ColStateParts = 22,
    ColEpochStart = 23,
    /// Map account_id to announce_account
    ColAccountAnnouncements = 24,
    /// Next block hashes in the sequence of the canonical chain blocks
    ColNextBlockHashes = 25,
    /// `LightClientBlock`s corresponding to the last final block of each completed epoch
    ColEpochLightClientBlocks = 26,
    ColReceiptIdToShardId = 27,
    // Deprecated.
    _ColNextBlockWithNewChunk = 28,
    // Deprecated.
    _ColLastBlockWithNewChunk = 29,
    /// Network storage:
    ///   When given edge is removed (or we didn't get any ping from it for a while), we remove it from our 'in memory'
    ///   view and persist into storage.
    ///
    ///   This is done, so that we prevent the attack, when someone tries to introduce the edge/peer again into the network,
    ///   but with the 'old' nonce.
    ///
    ///   When we write things to storage, we do it in groups (here they are called 'components') - this naming is a little bit
    ///   unfortunate, as the peers/edges that we persist don't need to be connected or form any other 'component' (in a graph theory sense).
    ///
    ///   Each such component gets a new identifier (here called 'nonce').
    ///
    ///   We store this info in the three columns below:
    ///     - LastComponentNonce: keeps info on what is the next identifier (nonce) that can be used.
    ///     - PeerComponent: keep information on mapping from the peer to the last component that it belonged to (so that if a new peer shows
    ///         up we know which 'component' to load)
    ///     - ComponentEdges: keep the info about the edges that were connecting these peers that were removed.

    /// Map each saved peer on disk with its component id (a.k.a. nonce).
    /// - *Rows*: peer_id
    /// - *Column type*:  (nonce) u64
    ColPeerComponent = 30,
    /// Map component id  (a.k.a. nonce) with all edges in this component.
    /// These are all the edges that were purged and persisted to disk at the same time.
    /// - *Rows*: nonce
    /// - *Column type*: `Vec<near_network::routing::Edge>`
    ColComponentEdges = 31,
    /// Biggest component id (a.k.a nonce) used.
    /// - *Rows*: single row (empty row name)
    /// - *Column type*: (nonce) u64
    ColLastComponentNonce = 32,
    /// Map of transactions
    /// - *Rows*: transaction hash
    /// - *Column type*: SignedTransaction
    ColTransactions = 33,
    ColChunkPerHeightShard = 34,
    /// Changes to key-values that we have recorded.
    ColStateChanges = 35,
    ColBlockRefCount = 36,
    ColTrieChanges = 37,
    /// Merkle tree of block hashes
    ColBlockMerkleTree = 38,
    ColChunkHashesByHeight = 39,
    /// Block ordinals.
    ColBlockOrdinal = 40,
    /// GC Count for each column
    ColGCCount = 41,
    /// All Outcome ids by block hash and shard id. For each shard it is ordered by execution order.
    ColOutcomeIds = 42,
    /// Deprecated
    _ColTransactionRefCount = 43,
    /// Heights of blocks that have been processed
    ColProcessedBlockHeights = 44,
    /// Receipts
    ColReceipts = 45,
    /// Precompiled machine code of the contract
    ColCachedContractCode = 46,
    /// Epoch validator information used for rpc purposes
    ColEpochValidatorInfo = 47,
    /// Header Hashes indexed by Height
    ColHeaderHashesByHeight = 48,
    /// State changes made by a chunk, used for splitting states
    ColStateChangesForSplitStates = 49,
}

// Do not move this line from enum DBCol
pub const NUM_COLS: usize = 50;

impl std::fmt::Display for DBCol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let desc = match self {
            Self::ColDbVersion => "db version",
            Self::ColBlockMisc => "miscellaneous block data",
            Self::ColBlock => "block data",
            Self::ColBlockHeader => "block header data",
            Self::ColBlockHeight => "block height",
            Self::ColState => "blockchain state",
            Self::ColChunkExtra => "extra information of trunk",
            Self::ColTransactionResult => "transaction results",
            Self::ColOutgoingReceipts => "outgoing receipts",
            Self::ColIncomingReceipts => "incoming receipts",
            Self::ColPeers => "peer information",
            Self::ColEpochInfo => "epoch information",
            Self::ColBlockInfo => "block information",
            Self::ColChunks => "chunks",
            Self::ColPartialChunks => "partial chunks",
            Self::ColBlocksToCatchup => "blocks need to apply chunks",
            Self::ColStateDlInfos => "blocks downloading",
            Self::ColChallengedBlocks => "challenged blocks",
            Self::ColStateHeaders => "state headers",
            Self::ColInvalidChunks => "invalid chunks",
            Self::ColBlockExtra => "extra block information",
            Self::ColBlockPerHeight => "hash of block per height",
            Self::ColStateParts => "state parts",
            Self::ColEpochStart => "epoch start",
            Self::ColAccountAnnouncements => "account announcements",
            Self::ColNextBlockHashes => "next block hash",
            Self::ColEpochLightClientBlocks => "epoch light client block",
            Self::ColReceiptIdToShardId => "receipt id to shard id",
            Self::_ColNextBlockWithNewChunk => "next block with new chunk (deprecated)",
            Self::_ColLastBlockWithNewChunk => "last block with new chunk (deprecated)",
            Self::ColPeerComponent => "peer components",
            Self::ColComponentEdges => "component edges",
            Self::ColLastComponentNonce => "last component nonce",
            Self::ColTransactions => "transactions",
            Self::ColChunkPerHeightShard => "hash of chunk per height and shard_id",
            Self::ColStateChanges => "key value changes",
            Self::ColBlockRefCount => "refcount per block",
            Self::ColTrieChanges => "trie changes",
            Self::ColBlockMerkleTree => "block merkle tree",
            Self::ColChunkHashesByHeight => "chunk hashes indexed by height_created",
            Self::ColBlockOrdinal => "block ordinal",
            Self::ColGCCount => "gc count",
            Self::ColOutcomeIds => "outcome ids",
            Self::_ColTransactionRefCount => "refcount per transaction (deprecated)",
            Self::ColProcessedBlockHeights => "processed block heights",
            Self::ColReceipts => "receipts",
            Self::ColCachedContractCode => "cached code",
            Self::ColEpochValidatorInfo => "epoch validator info",
            Self::ColHeaderHashesByHeight => "header hashes indexed by their height",
            Self::ColStateChangesForSplitStates => {
                "state changes indexed by block hash and shard id"
            }
        };
        write!(formatter, "{}", desc)
    }
}

impl DBCol {
    pub fn is_rc(&self) -> bool {
        IS_COL_RC[*self as usize]
    }
}

// List of columns for which GC should be implemented
pub static SHOULD_COL_GC: [bool; NUM_COLS] = {
    let mut col_gc = [true; NUM_COLS];
    col_gc[DBCol::ColDbVersion as usize] = false; // DB version is unrelated to GC
    col_gc[DBCol::ColBlockMisc as usize] = false;
    // TODO #3488 remove
    col_gc[DBCol::ColBlockHeader as usize] = false; // header sync needs headers
    col_gc[DBCol::ColGCCount as usize] = false; // GC count it self isn't GCed
    col_gc[DBCol::ColBlockHeight as usize] = false; // block sync needs it + genesis should be accessible
    col_gc[DBCol::ColPeers as usize] = false; // Peers is unrelated to GC
    col_gc[DBCol::ColBlockMerkleTree as usize] = false;
    col_gc[DBCol::ColAccountAnnouncements as usize] = false;
    col_gc[DBCol::ColEpochLightClientBlocks as usize] = false;
    col_gc[DBCol::ColPeerComponent as usize] = false; // Peer related info doesn't GC
    col_gc[DBCol::ColLastComponentNonce as usize] = false;
    col_gc[DBCol::ColComponentEdges as usize] = false;
    col_gc[DBCol::ColBlockOrdinal as usize] = false;
    col_gc[DBCol::ColEpochInfo as usize] = false; // https://github.com/nearprotocol/nearcore/pull/2952
    col_gc[DBCol::ColEpochValidatorInfo as usize] = false; // https://github.com/nearprotocol/nearcore/pull/2952
    col_gc[DBCol::ColEpochStart as usize] = false; // https://github.com/nearprotocol/nearcore/pull/2952
    col_gc[DBCol::ColCachedContractCode as usize] = false;
    col_gc
};

// List of columns for which GC may not be executed even in fully operational node

pub static SKIP_COL_GC: [bool; NUM_COLS] = {
    let mut col_gc = [false; NUM_COLS];
    // A node may never restarted
    col_gc[DBCol::ColStateHeaders as usize] = true;
    // True until #2515
    col_gc[DBCol::ColStateParts as usize] = true;
    col_gc
};

// List of reference counted columns

pub static IS_COL_RC: [bool; NUM_COLS] = {
    let mut col_rc = [false; NUM_COLS];
    col_rc[DBCol::ColState as usize] = true;
    col_rc[DBCol::ColTransactions as usize] = true;
    col_rc[DBCol::ColReceipts as usize] = true;
    col_rc[DBCol::ColReceiptIdToShardId as usize] = true;
    col_rc
};

pub const HEAD_KEY: &[u8; 4] = b"HEAD";
pub const TAIL_KEY: &[u8; 4] = b"TAIL";
pub const CHUNK_TAIL_KEY: &[u8; 10] = b"CHUNK_TAIL";
pub const FORK_TAIL_KEY: &[u8; 9] = b"FORK_TAIL";
pub const HEADER_HEAD_KEY: &[u8; 11] = b"HEADER_HEAD";
pub const FINAL_HEAD_KEY: &[u8; 10] = b"FINAL_HEAD";
pub const LATEST_KNOWN_KEY: &[u8; 12] = b"LATEST_KNOWN";
pub const LARGEST_TARGET_HEIGHT_KEY: &[u8; 21] = b"LARGEST_TARGET_HEIGHT";
pub const VERSION_KEY: &[u8; 7] = b"VERSION";
pub const GENESIS_JSON_HASH_KEY: &[u8; 17] = b"GENESIS_JSON_HASH";
pub const GENESIS_STATE_ROOTS_KEY: &[u8; 19] = b"GENESIS_STATE_ROOTS";

pub struct DBTransaction {
    pub ops: Vec<DBOp>,
}

pub enum DBOp {
    Insert { col: DBCol, key: Vec<u8>, value: Vec<u8> },
    UpdateRefcount { col: DBCol, key: Vec<u8>, value: Vec<u8> },
    Delete { col: DBCol, key: Vec<u8> },
    DeleteAll { col: DBCol },
}

impl DBTransaction {
    pub fn put<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, col: DBCol, key: K, value: V) {
        self.ops.push(DBOp::Insert {
            col,
            key: key.as_ref().to_owned(),
            value: value.as_ref().to_owned(),
        });
    }

    pub fn update_refcount<K: AsRef<[u8]>, V: AsRef<[u8]>>(
        &mut self,
        col: DBCol,
        key: K,
        value: V,
    ) {
        self.ops.push(DBOp::UpdateRefcount {
            col,
            key: key.as_ref().to_owned(),
            value: value.as_ref().to_owned(),
        });
    }

    pub fn delete<K: AsRef<[u8]>>(&mut self, col: DBCol, key: K) {
        self.ops.push(DBOp::Delete { col, key: key.as_ref().to_owned() });
    }

    pub fn delete_all(&mut self, col: DBCol) {
        self.ops.push(DBOp::DeleteAll { col });
    }
}

pub struct FileDB {
    db: RwLock<Vec<hashbrown::HashMap<Vec<u8>, Vec<u8>>>>,
}

pub trait Database: Sync + Send {
    fn transaction(&self) -> DBTransaction {
        DBTransaction { ops: Vec::new() }
    }
    fn get(&self, col: DBCol, key: &[u8]) -> Option<Vec<u8>>;
    fn iter<'a>(&'a self, column: DBCol) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;
    fn iter_without_rc_logic<'a>(
        &'a self,
        column: DBCol,
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;
    fn iter_prefix<'a>(
        &'a self,
        col: DBCol,
        key_prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;
    fn write(&self, batch: DBTransaction) -> ();
}

impl Database for FileDB {
    fn get(&self, col: DBCol, key: &[u8]) -> Option<Vec<u8>> {
        let result = self.db.read().unwrap()[col as usize].get(key).cloned();
        FileDB::get_with_rc_logic(col, result)
    }

    fn iter<'a>(&'a self, col: DBCol) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        let iterator = self.iter_without_rc_logic(col);
        FileDB::iter_with_rc_logic(col, iterator)
    }

    fn iter_without_rc_logic<'a>(
        &'a self,
        col: DBCol,
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        let iterator = self.db.read().unwrap()[col as usize]
            .clone()
            .into_iter()
            .map(|(k, v)| (k.into_boxed_slice(), v.into_boxed_slice()));
        Box::new(iterator)
    }

    fn iter_prefix<'a>(
        &'a self,
        col: DBCol,
        key_prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        FileDB::iter_with_rc_logic(
            col,
            self.iter(col).filter(move |(key, _value)| key.starts_with(key_prefix)),
        )
    }

    fn write(&self, transaction: DBTransaction) {
        let mut db = self.db.write().unwrap();
        for op in transaction.ops {
            match op {
                DBOp::Insert { col, key, value } => {
                    db[col as usize].insert(key, value);
                }
                DBOp::UpdateRefcount { col, key, value } => {
                    let mut val = db[col as usize].get(&key).cloned().unwrap_or_default();
                    merge_refcounted_records(&mut val, &value);
                    if !val.is_empty() {
                        db[col as usize].insert(key, val);
                    } else {
                        db[col as usize].remove(&key);
                    }
                }
                DBOp::Delete { col, key } => {
                    db[col as usize].remove(&key);
                }
                DBOp::DeleteAll { col } => db[col as usize].clear(),
            };
        }
    }
}

impl FileDB {
    pub fn new() -> Self {
        let db: Vec<_> = (0..NUM_COLS).map(|_| hashbrown::HashMap::new()).collect();
        Self { db: RwLock::new(db) }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::DBCol::ColState;
    use crate::{create_store};

    #[test]
    fn test_clear_column() {
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
        {
            let mut store_update = store.store_update();
            store_update.delete_all(ColState);
            store_update.commit().unwrap();
        }
        assert_eq!(store.get(ColState, &[1]), None);
    }
}
