use sp_std::prelude::*;

pub type CallIndex = u32;
pub type ShardId = u32;
pub type SecretId = u32;

pub type BlockNumber = u32;
pub type PublicKey = [u8; 32];

pub type EncodedCall = Vec<u8>;
pub type ContractName = Vec<u8>;

pub type Balance = u32;
pub type CryptoHash = [u8; 32];
pub type PoASingature = [u8; 64];

pub use borsh::{BorshSerialize, BorshDeserialize};

pub type Bytes = Vec<u8>;

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Call {
	pub origin_public_key: PublicKey, // sender OR system
    pub receipt_public_key: PublicKey, // contract addr OR another user 
    pub encrypted_egress: bool, // 

    pub transaction_action: u8, //  view function, call function,

    // in sync with BalanceOf
    pub amount: Option<Balance>,
    pub contract_name: Option<Bytes>,
    pub method: Option<Bytes>,
    pub args: Option<Bytes>,
    pub wasm_code: Option<Bytes>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Calls {
	pub ops: Vec<Call>,
	pub shard_id: ShardId,
	pub block_number: Option<BlockNumber>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
pub struct Outcome {
    pub view_result_log: Vec<Bytes>,
    pub view_result: Option<Bytes>,
    pub view_error: Option<Bytes>,
    pub outcome_logs: Vec<Bytes>,
    pub outcome_tokens_burnt: Balance,
    pub outcome_status: Option<Bytes>,

    pub encrypted: Option<Bytes>,
}


#[derive(BorshSerialize, BorshDeserialize, Default, Debug)]
pub struct Outcomes {
    pub ops: Vec<Outcome>,
    pub call_index: CallIndex,
    pub signature: Bytes,
    pub state_root: CryptoHash,
}
