use crate::{
    config::RuntimeConfig,
    contract_runtime::{CryptoHash, Balance, BlockNumber, Gas},
};
use std::sync::Arc;

#[derive(Debug)]
pub struct ApplyState {
    /// Currently building block height.
    pub block_number: BlockNumber,
    /// Prev block hash
    pub prev_block_hash: CryptoHash,
    /// Current block hash
    pub block_hash: CryptoHash,
    /// Price for the gas.
    pub gas_price: Balance,
    /// The current block timestamp (number of non-leap-nanoseconds since January 1, 1970 0:00:00 UTC).
    pub block_timestamp: u64,
    /// Gas limit for a given chunk.
    /// If None is given, assumes there is no gas limit.
    pub gas_limit: Option<Gas>,
    /// Current random seed (from current block vrf output).
    pub random_seed: CryptoHash,
    /// The Runtime config to use for the current transition.
    pub config: Arc<RuntimeConfig>,
}
