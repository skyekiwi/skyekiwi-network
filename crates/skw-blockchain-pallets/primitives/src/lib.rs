#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;

mod util;
pub use util::*;

pub type CallIndex = u64;
pub type ShardId = u64;
pub type SecretId = u64;

pub type PublicKey = [u8; 32];

pub type EncodedCall = Vec<u8>;
pub type ContractName = Vec<u8>;

pub type Gas = u64;
pub type Balance = u128;
pub type CryptoHash = [u8; 32];

pub use borsh::{BorshSerialize, BorshDeserialize};

pub type Bytes = Vec<u8>;

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct InputParams {
    pub origin: Option<Bytes>,
	pub origin_public_key: Option<[u8; 32]>,	
    pub encrypted_egress: bool,

    pub transaction_action: Bytes,
    pub receiver: Bytes,
    pub amount: Option<u32>,
    pub wasm_blob_path: Option<Bytes>,
    pub method: Option<Bytes>,
    pub args: Option<Bytes>,
    pub to: Option<Bytes>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Input {
	pub ops: Vec<InputParams>,
	pub shard_id: ShardId,
	pub block_number: Option<ShardId>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct InterfaceOutcome {
    pub view_result_log: Vec<Bytes>,
    pub view_result: Vec<u8>,
    pub outcome_logs: Vec<Bytes>,
    pub outcome_receipt_ids: Vec<CryptoHash>,
    pub outcome_gas_burnt: Gas,
    pub outcome_tokens_burnt: Balance,
    pub outcome_executor_id: Bytes,
    pub outcome_status: Option<Vec<u8>>,
}

pub type StatePatch = Vec<u8>;

#[derive(BorshSerialize, BorshDeserialize, Default, Debug)]
pub struct Outputs {
    pub ops: Vec<InterfaceOutcome>,
    pub state_root: CryptoHash,
    pub state_patch: StatePatch,
}

