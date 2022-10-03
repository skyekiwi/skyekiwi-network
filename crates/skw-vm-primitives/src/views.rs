use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, Deserialize};
use crate::contract_runtime::{BlockNumber, CryptoHash};

/// State for the view call.
#[derive(Debug)]
pub struct ViewApplyState {
    /// Currently building block height.
    pub block_number: BlockNumber,
    /// Prev block hash
    pub prev_block_hash: CryptoHash,
    /// Currently building block hash
    pub block_hash: CryptoHash,
    /// The current block timestamp (number of non-leap-nanoseconds since January 1, 1970 0:00:00 UTC).
    pub block_timestamp: u64,
}

/// Set of serialized TrieNodes that are encoded in base64. Represent proof of inclusion of some TrieNode in the MerkleTrie.
pub type TrieProofPath = Vec<String>;

/// Item of the state, key and value are serialized in base64 and proof for inclusion of given state item.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct StateItem {
    pub key: String,
    pub value: String,
    pub proof: TrieProofPath,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ViewStateResult {
    pub values: Vec<StateItem>,
    pub proof: TrieProofPath,
}
