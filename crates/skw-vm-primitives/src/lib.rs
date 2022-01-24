pub mod account_id;
pub mod config;
pub mod trie_key;
pub mod errors;
pub mod fees;
pub mod profile;
pub mod receipt;
pub mod transaction;
pub mod serialize;
pub mod logging;
pub mod apply_state;
pub mod utils;
pub mod views;
pub mod state_record;
pub mod account;
pub mod challenge;
pub mod test_utils;

pub use num_rational;
pub use borsh;

pub mod crypto;

pub mod contract_runtime {
    use sha2::Digest;
    pub use crate::account_id::AccountId;
    use crate::receipt::{Receipt};
    use crate::trie_key::TrieKey;
    use borsh::{BorshDeserialize, BorshSerialize};
    use serde::{Serialize, Deserialize};
    use crate::crypto::PublicKey;
    use crate::serialize::u128_dec_format;

    pub type CryptoHash = [u8; 32];
    pub type MerkleHash = CryptoHash;
    pub type StateRoot = CryptoHash;
    pub type ProtocolVersion = u32;
    pub type BlockNumber = u64;
    pub type EpochHeight = u64;
    pub type Balance = u128;
    pub type StorageUsage = u64;
    pub type Gas = u64;
    pub type Nonce = u64;
    pub struct ContractCode {
        pub code: Vec<u8>,
        pub hash: CryptoHash,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct AccountInfo {
        pub account_id: AccountId,
        pub public_key: PublicKey,
        #[serde(with = "u128_dec_format")]
        pub amount: Balance,
    }

    // near_primitives::errors::RuntimeError;

    impl ContractCode {
        pub fn new(code: &[u8]) -> ContractCode {
            ContractCode {
                code: code.to_vec(),
                hash: hash_bytes(&code)
            }
        }
    }

    pub fn hash_bytes(bytes: &[u8]) -> CryptoHash {
        sha2::Sha256::digest(bytes).into()
    }


    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    pub enum StateChangeCause {
        /// A type of update that does not get finalized. Used for verification and execution of
        /// immutable smart contract methods. Attempt fo finalize a `TrieUpdate` containing such
        /// change will lead to panic.
        NotWritableToDisk,
        /// A type of update that is used to mark the initial storage update, e.g. during genesis
        /// or in tests setup.
        InitialState,
        /// Processing of a transaction.
        TransactionProcessing { tx_hash: CryptoHash },
        /// Before the receipt is going to be processed, inputs get drained from the state, which
        /// causes state modification.
        ActionReceiptProcessingStarted { receipt_hash: CryptoHash },
        /// Computation of gas reward.
        ActionReceiptGasReward { receipt_hash: CryptoHash },
        /// Processing of a receipt.
        ReceiptProcessing { receipt_hash: CryptoHash },
        /// The given receipt was postponed. This is either a data receipt or an action receipt.
        /// A `DataReceipt` can be postponed if the corresponding `ActionReceipt` is not received yet,
        /// or other data dependencies are not satisfied.
        /// An `ActionReceipt` can be postponed if not all data dependencies are received.
        PostponedReceipt { receipt_hash: CryptoHash },
        /// Updated delayed receipts queue in the state.
        /// We either processed previously delayed receipts or added more receipts to the delayed queue.
        UpdatedDelayedReceipts,
        /// State change that happens when we update validator accounts. Not associated with with any
        /// specific transaction or receipt.
        ValidatorAccountsUpdate,
        /// State change that is happens due to migration that happens in first block of an epoch
        /// after protocol upgrade
        Migration,
        /// State changes for building states for re-sharding
        Resharding,
    }

    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    pub struct RawStateChange {
        pub cause: StateChangeCause,
        pub data: Option<Vec<u8>>,
    }

    /// List of committed changes with a cause for a given TrieKey
    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    pub struct RawStateChangesWithTrieKey {
        pub trie_key: crate::trie_key::TrieKey,
        pub changes: Vec<RawStateChange>,
    }

    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    pub struct ConsolidatedStateChange {
        pub trie_key: TrieKey,
        pub value: Option<Vec<u8>>,
    }

    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    pub struct StateChangesForSplitStates {
        pub changes: Vec<ConsolidatedStateChange>,
        // we need to store deleted receipts here because StateChanges will only include
        // trie keys for removed values and account information can not be inferred from
        // trie key for delayed receipts
        pub processed_delayed_receipts: Vec<Receipt>,
    }

    impl StateChangesForSplitStates {
        pub fn from_raw_state_changes(
            changes: &[RawStateChangesWithTrieKey],
            processed_delayed_receipts: Vec<Receipt>,
        ) -> Self {
            let changes = changes
                .iter()
                .map(|RawStateChangesWithTrieKey { trie_key, changes }| {
                    let value = changes.last().expect("state_changes must not be empty").data.clone();
                    ConsolidatedStateChange { trie_key: trie_key.clone(), value }
                })
                .collect();
            Self { changes, processed_delayed_receipts }
        }
    }

    pub type RawStateChanges = std::collections::BTreeMap<Vec<u8>, RawStateChangesWithTrieKey>;

    #[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
    #[derive(PartialEq, Eq, Clone, Debug, BorshSerialize, BorshDeserialize, serde::Serialize)]
    pub struct StateRootNode {
        /// in Nightshade, data is the serialized TrieNodeWithSize
        pub data: Vec<u8>,
        /// in Nightshade, memory_usage is a field of TrieNodeWithSize
        pub memory_usage: u64,
    }

    impl StateRootNode {
        pub fn empty() -> Self {
            StateRootNode { data: vec![], memory_usage: 0 }
        }
    }
}
