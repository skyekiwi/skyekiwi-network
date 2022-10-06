pub mod public_key;
pub mod account_id;

pub use public_key::{PublicKey, ParsePublicKeyError};
pub use account_id::AccountId;

#[cfg(not(target_arch = "wasm32"))]
pub use skw_vm_primitives::fees::RuntimeFeesConfig;

pub type CryptoHash = [u8; 32];
/// Hash used by a struct implementing the Merkle tree.
#[cfg(not(target_arch = "wasm32"))]
pub type MerkleHash = CryptoHash;
/// StorageUsage is used to count the amount of storage used by a contract.
pub type StorageUsage = u64;
/// StorageUsageChange is used to count the storage usage within a single contract call.
pub type StorageUsageChange = i64;
/// Nonce for transactions.
pub type Nonce = u64;
/// Number of the block.
pub type BlockNumber = u64;

pub type Gas = u64;

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

#[cfg(not(target_arch = "wasm32"))]
pub use skw_vm_host::types::{PromiseResult as VmPromiseResult, ReturnData};

//* Types from skw_vm_host
pub type PromiseIndex = u64;
pub type ReceiptIndex = u64;
pub type IteratorIndex = u64;

/// When there is a callback attached to one or more contract calls the execution results of these
/// calls are available to the contract invoked through the callback.
#[derive(Debug, PartialEq)]
pub enum PromiseResult {
    /// Current version of the protocol never returns `PromiseResult::NotReady`.
    NotReady,
    Successful(Vec<u8>),
    Failed,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<PromiseResult> for VmPromiseResult {
    fn from(p: PromiseResult) -> Self {
        match p {
            PromiseResult::NotReady => Self::NotReady,
            PromiseResult::Successful(v) => Self::Successful(v),
            PromiseResult::Failed => Self::Failed,
        }
    }
}

/// All error variants which can occur with promise results.
#[non_exhaustive]
pub enum PromiseError {
    /// Promise result failed.
    Failed,
    /// Current version of the protocol never returns this variant.
    NotReady,
}