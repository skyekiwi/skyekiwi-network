use sp_std::prelude::*;

pub type CallIndex = u64;
pub type ShardId = u64;
pub type SecretId = u64;

pub type BlockNumber = u32;
pub type PublicKey = [u8; 32];

pub type EncodedCall = Vec<u8>;
pub type ContractName = Vec<u8>;

pub type Balance = u32;
pub type CryptoHash = [u8; 32];

pub use borsh::{BorshSerialize, BorshDeserialize};

pub type Bytes = Vec<u8>;

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Call{
	pub origin_public_key: PublicKey,
    pub receipt_public_key: PublicKey,
    pub encrypted_egress: bool,

    pub transaction_action: u8,

    // in sync with BalanceOf
    pub amount: Option<Balance>,
    pub wasm_blob_path: Option<Bytes>,
    pub method: Option<Bytes>,
    pub args: Option<Bytes>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Calls{
	pub ops: Vec<Call>,
	pub shard_id: ShardId,
	pub block_number: Option<BlockNumber>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Outcome{
    pub view_result_log: Vec<Bytes>,
    pub view_result: Option<Vec<u8>>,
    pub view_error: Option<Vec<u8>>,
    pub outcome_logs: Vec<Bytes>,
    pub outcome_receipt_ids: Vec<CryptoHash>,
    pub outcome_tokens_burnt: Balance,
    pub outcome_executor_id: Bytes,
    pub outcome_status: Option<Vec<u8>>,
}

pub type StatePatch = Vec<u8>;

#[derive(BorshSerialize, BorshDeserialize, Default, Debug)]
pub struct Outcomes {
    pub ops: Vec<Outcome>,
    pub state_root: CryptoHash,
    pub state_patch: StatePatch,
}
