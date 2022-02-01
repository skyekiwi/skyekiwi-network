use byteorder::{LittleEndian, WriteBytesExt};

use crate::contract_runtime::{hash_bytes, CryptoHash};
use crate::receipt::Receipt;
use crate::transaction::SignedTransaction;
use std::mem::size_of;

/// Creates a new Receipt ID from a given signed transaction and a block hash.
/// This method is backward compatible, so it takes the current protocol version.
pub fn create_receipt_id_from_transaction(
    signed_transaction: &SignedTransaction,
) -> CryptoHash {
    create_nonce_with_nonce(
        &signed_transaction.get_hash(),
        0,
    )
}

/// Creates a new Receipt ID from a given receipt, a block hash and a new receipt index.
/// This method is backward compatible, so it takes the current protocol version.
pub fn create_receipt_id_from_receipt(
    receipt: &Receipt,
    receipt_index: usize,
) -> CryptoHash {
    create_nonce_with_nonce(
        &receipt.receipt_id,
        receipt_index as u64,
    )
}

/// Creates a new action_hash from a given receipt, a block hash and an action index.
/// This method is backward compatible, so it takes the current protocol version.
pub fn create_action_hash(
    receipt: &Receipt,
    action_index: usize,
) -> CryptoHash {
    // Action hash uses the same input as a new receipt ID, so to avoid hash conflicts we use the
    // salt starting from the `u64` going backward.
    let salt = u64::max_value() - action_index as u64;
    create_nonce_with_nonce(&receipt.receipt_id, salt)
}

/// Creates a new `data_id` from a given action hash, a block hash and a data index.
/// This method is backward compatible, so it takes the current protocol version.
pub fn create_data_id(
    action_hash: &CryptoHash,
    data_index: usize,
) -> CryptoHash {
    create_nonce_with_nonce(
        action_hash,
        data_index as u64,
    )
}

/// Creates a unique random seed to be provided to `VMContext` from a give `action_hash` and
/// a given `random_seed`.
/// This method is backward compatible, so it takes the current protocol version.
pub fn create_random_seed(
    action_hash: CryptoHash,
    random_seed: CryptoHash,
) -> Vec<u8> {
    let res = {
        // Generates random seed from random_seed and action_hash.
        // Since every action hash is unique, the seed will be unique per receipt and even
        // per action within a receipt.
        let mut bytes: Vec<u8> =
            Vec::with_capacity(size_of::<CryptoHash>() + size_of::<CryptoHash>());
        bytes.extend_from_slice(action_hash.as_ref());
        bytes.extend_from_slice(random_seed.as_ref());
        hash_bytes(&bytes)
    };
    res.as_ref().to_vec()
}

fn create_nonce_with_nonce(base: &CryptoHash, salt: u64) -> CryptoHash {
    let mut nonce: Vec<u8> = base.as_ref().to_owned();
    nonce.extend(index_to_bytes(salt));
    hash_bytes(&nonce)
}

pub fn index_to_bytes(index: u64) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.write_u64::<LittleEndian>(index).expect("writing to bytes failed");
    bytes
}

