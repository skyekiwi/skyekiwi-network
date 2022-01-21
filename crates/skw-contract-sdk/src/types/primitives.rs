#[cfg(not(target_arch = "wasm32"))]
use skw_vm_primitives::contract_runtime::CryptoHash;

#[cfg(not(target_arch = "wasm32"))]
pub use skw_vm_primitives::fees::RuntimeFeesConfig;


//* Type aliases from near_primitives_core


// TODO: we might get the next three entries removed
/// Hash used by a struct implementing the Merkle tree.
#[cfg(not(target_arch = "wasm32"))]
pub type MerkleHash = CryptoHash;
/// Validator identifier in current group.
pub type ValidatorId = u64;
/// Mask which validators participated in multi sign.
pub type ValidatorMask = Vec<bool>;

/// StorageUsage is used to count the amount of storage used by a contract.
pub type StorageUsage = u64;
/// StorageUsageChange is used to count the storage usage within a single contract call.
pub type StorageUsageChange = i64;
/// Nonce for transactions.
pub type Nonce = u64;
/// Number of the block.
pub type BlockNumber = u64;
/// Height of the epoch.
pub type EpochHeight = u64;

// DIFF: dont need shard but keep it now; SecretKeepers act similar to shards
// /// Shard index, from 0 to NUM_SHARDS - 1.
// pub type ShardId = u64;


/// Balance is a type for storing amounts of tokens, specified in yoctoNEAR.
pub type Balance = u128;

/// Number of blocks in current group.
pub type NumBlocks = u64;
/// Number of shards in current group.
pub type NumShards = u64;
/// Number of seats of validators (block producer or hidden ones) in current group (settlement).
pub type NumSeats = u64;
/// Block height delta that measures the difference between `BlockHeight`s.
pub type BlockHeightDelta = u64;

pub type GCCount = u64;

pub type PromiseId = Vec<usize>;

pub type ProtocolVersion = u32;
