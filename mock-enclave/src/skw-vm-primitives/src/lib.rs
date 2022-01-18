pub mod account_id;
pub mod config;
pub mod db_key;
pub mod errors;
pub mod fees;
pub mod profile;
pub mod receipt;
pub mod transaction;
pub mod serialize;
pub mod logging;

pub mod contract_runtime {
    use sha2::Digest;
    pub use crate::account_id::AccountId;
    
    pub type CryptoHash = [u8; 32];
    pub type ProtocolVersion = u32;
    pub type BlockHeight = u64;
    pub type EpochHeight = u64;
    pub type Balance = u128;
    pub type StorageUsage = u64;
    pub type Gas = u64;
    pub type Nonce = u64;
    pub struct ContractCode {
        pub code: Vec<u8>,
        pub hash: CryptoHash,
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

}

