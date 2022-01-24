use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::crypto::{PublicKey, Signature};

use crate::contract_runtime::{CryptoHash, hash_bytes, AccountId, Balance, Gas, Nonce};
use crate::serialize::{base64_format, u128_dec_format_compatible};
use crate::profile::ProfileData;

use crate::errors::TxExecutionError;

// // use crate::account::AccessKey;
// // use crate::merkle::MerklePath;

pub type LogEntry = String;

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Transaction {
    /// An account on which behalf transaction is signed
    pub signer_id: AccountId,
    /// A public key of the access key which was used to sign an account.
    /// Access key holds permissions for calling certain kinds of actions.
    pub public_key: PublicKey,
    /// Nonce is used to determine order of transaction in the pool.
    /// It increments for a combination of `signer_id` and `public_key`
    pub nonce: Nonce,
    /// Receiver account for this transaction
    pub receiver_id: AccountId,
    /// The hash of the block in the blockchain on top of which the given transaction is valid
    pub block_hash: CryptoHash,
    /// A list of actions to be applied
    pub actions: Vec<Action>,
}

impl Transaction {
    /// Computes a hash of the transaction for signing and size of serialized transaction
    pub fn get_hash_and_size(&self) -> (CryptoHash, u64) {
        let bytes = self.try_to_vec().expect("Failed to deserialize");
        (hash_bytes(&bytes), bytes.len() as u64)
    }
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Action {
    /// Sets a Wasm code to a receiver_id
    CreateAccount(CreateAccountAction),
    Transfer(TransferAction),
    DeployContract(DeployContractAction),
    FunctionCall(FunctionCallAction),
    DeleteAccount(DeleteAccountAction),
}

impl Action {
    pub fn get_prepaid_gas(&self) -> Gas {
        match self {
            Action::FunctionCall(a) => a.gas,
            _ => 0,
        }
    }
    pub fn get_deposit_balance(&self) -> Balance {
        match self {
            Action::FunctionCall(a) => a.deposit,
            _ => 0,
        }
    }
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct CreateAccountAction {}

impl From<CreateAccountAction> for Action {
    fn from(create_account_action: CreateAccountAction) -> Self {
        Self::CreateAccount(create_account_action)
    }
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct DeleteAccountAction {
    pub beneficiary_id: AccountId,
}

impl From<DeleteAccountAction> for Action {
    fn from(delete_account_action: DeleteAccountAction) -> Self {
        Self::DeleteAccount(delete_account_action)
    }
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct TransferAction {
    #[serde(with = "u128_dec_format_compatible")]
    pub deposit: Balance,
}

impl From<TransferAction> for Action {
    fn from(transfer_action: TransferAction) -> Self {
        Self::Transfer(transfer_action)
    }
}

/// Deploy contract action
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DeployContractAction {
    /// WebAssembly binary
    #[serde(with = "base64_format")]
    pub code: Vec<u8>,
}

impl From<DeployContractAction> for Action {
    fn from(deploy_contract_action: DeployContractAction) -> Self {
        Self::DeployContract(deploy_contract_action)
    }
}

impl fmt::Debug for DeployContractAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeployContractAction")
            .field("code", &format_args!("{}", crate::logging::pretty_utf8(&self.code)))
            .finish()
    }
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct FunctionCallAction {
    pub method_name: String,
    #[serde(with = "base64_format")]
    pub args: Vec<u8>,
    pub gas: Gas,
    #[serde(with = "u128_dec_format_compatible")]
    pub deposit: Balance,
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Eq, Debug, Clone)]
#[borsh_init(init)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Signature,
    #[borsh_skip]
    hash: CryptoHash,
    #[borsh_skip]
    size: u64,
}

impl SignedTransaction {
    pub fn new(signature: Signature, transaction: Transaction) -> Self {
        let mut signed_tx =
            Self { signature, transaction, hash: CryptoHash::default(), size: u64::default() };
        signed_tx.init();
        signed_tx
    }

    pub fn init(&mut self) {
        let (hash, size) = self.transaction.get_hash_and_size();
        self.hash = hash;
        self.size = size;
    }

    pub fn get_hash(&self) -> CryptoHash {
        self.hash
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }
}

impl From<FunctionCallAction> for Action {
    fn from(function_call_action: FunctionCallAction) -> Self {
        Self::FunctionCall(function_call_action)
    }
}

impl fmt::Debug for FunctionCallAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionCallAction")
            .field("method_name", &format_args!("{}", &self.method_name))
            .field("args", &format_args!("{}", crate::logging::pretty_utf8(&self.args)))
            .field("gas", &format_args!("{}", &self.gas))
            .field("deposit", &format_args!("{}", &self.deposit))
            .finish()
    }
}

impl Hash for SignedTransaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state)
    }
}

impl PartialEq for SignedTransaction {
    fn eq(&self, other: &SignedTransaction) -> bool {
        self.hash == other.hash && self.signature == other.signature
    }
}

impl Borrow<CryptoHash> for SignedTransaction {
    fn borrow(&self) -> &CryptoHash {
        &self.hash
    }
}

/// The status of execution for a transaction or a receipt.
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone)]
pub enum ExecutionStatus {
    /// The execution is pending or unknown.
    Unknown,
    /// The execution has failed with the given execution error.
    Failure(TxExecutionError),
    /// The final action succeeded and returned some value or an empty vec.
    SuccessValue(Vec<u8>),
    /// The final action of the receipt returned a promise or the signed transaction was converted
    /// to a receipt. Contains the receipt_id of the generated receipt.
    SuccessReceiptId(CryptoHash),
}

impl fmt::Debug for ExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionStatus::Unknown => f.write_str("Unknown"),
            ExecutionStatus::Failure(e) => f.write_fmt(format_args!("Failure({})", e)),
            ExecutionStatus::SuccessValue(v) => {
                f.write_fmt(format_args!("SuccessValue({})", crate::logging::pretty_utf8(v)))
            }
            ExecutionStatus::SuccessReceiptId(receipt_id) => {
                f.write_fmt(format_args!("SuccessReceiptId({:?})", receipt_id))
            }
        }
    }
}

impl Default for ExecutionStatus {
    fn default() -> Self {
        ExecutionStatus::Unknown
    }
}

/// ExecutionOutcome for proof. Excludes logs and metadata
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone)]
struct PartialExecutionOutcome {
    pub receipt_ids: Vec<CryptoHash>,
    pub gas_burnt: Gas,
    pub tokens_burnt: Balance,
    pub executor_id: AccountId,
    pub status: PartialExecutionStatus,
}

impl From<&ExecutionOutcome> for PartialExecutionOutcome {
    fn from(outcome: &ExecutionOutcome) -> Self {
        Self {
            receipt_ids: outcome.receipt_ids.clone(),
            gas_burnt: outcome.gas_burnt,
            tokens_burnt: outcome.tokens_burnt,
            executor_id: outcome.executor_id.clone(),
            status: outcome.status.clone().into(),
        }
    }
}

/// ExecutionStatus for proof. Excludes failure debug info.
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub enum PartialExecutionStatus {
    Unknown,
    Failure,
    SuccessValue(Vec<u8>),
    SuccessReceiptId(CryptoHash),
}

impl From<ExecutionStatus> for PartialExecutionStatus {
    fn from(status: ExecutionStatus) -> PartialExecutionStatus {
        match status {
            ExecutionStatus::Unknown => PartialExecutionStatus::Unknown,
            ExecutionStatus::Failure(_) => PartialExecutionStatus::Failure,
            ExecutionStatus::SuccessValue(value) => PartialExecutionStatus::SuccessValue(value),
            ExecutionStatus::SuccessReceiptId(id) => PartialExecutionStatus::SuccessReceiptId(id),
        }
    }
}

/// Execution outcome for one signed transaction or one receipt.
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, smart_default::SmartDefault, Eq)]
pub struct ExecutionOutcome {
    /// Logs from this transaction or receipt.
    pub logs: Vec<LogEntry>,
    /// Receipt IDs generated by this transaction or receipt.
    pub receipt_ids: Vec<CryptoHash>,
    /// The amount of the gas burnt by the given transaction or receipt.
    pub gas_burnt: Gas,
    /// The amount of tokens burnt corresponding to the burnt gas amount.
    /// This value doesn't always equal to the `gas_burnt` multiplied by the gas price, because
    /// the prepaid gas price might be lower than the actual gas price and it creates a deficit.
    pub tokens_burnt: Balance,
    /// The id of the account on which the execution happens. For transaction this is signer_id,
    /// for receipt this is receiver_id.
    #[default("test".parse().unwrap())]
    pub executor_id: AccountId,
    /// Execution status. Contains the result in case of successful execution.
    /// NOTE: Should be the latest field since it contains unparsable by light client
    /// ExecutionStatus::Failure
    pub status: ExecutionStatus,
    /// Execution metadata, versioned 
	/// DIFF: changed to ProfileData
    pub profile_data: Option<ProfileData>,
}

impl ExecutionOutcome {
    pub fn to_hashes(&self) -> Vec<CryptoHash> {
        let mut result = vec![hash_bytes(
            &PartialExecutionOutcome::from(self).try_to_vec().expect("Failed to serialize"),
        )];
        for log in self.logs.iter() {
            result.push(hash_bytes(log.as_bytes()));
        }
        result
    }
}

impl fmt::Debug for ExecutionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionOutcome")
            .field("logs", &format_args!("{}", crate::logging::pretty_vec(&self.logs)))
            .field("receipt_ids", &format_args!("{}", crate::logging::pretty_vec(&self.receipt_ids)))
            .field("burnt_gas", &self.gas_burnt)
            .field("tokens_burnt", &self.tokens_burnt)
            .field("status", &self.status)
            .field("profile_data", &self.profile_data)
            .finish()
    }
}

/// Execution outcome with the identifier.
/// For a signed transaction, the ID is the hash of the transaction.
/// For a receipt, the ID is the receipt ID.
#[derive(PartialEq, Clone, Default, Debug, BorshSerialize, BorshDeserialize, Eq)]
pub struct ExecutionOutcomeWithId {
    /// The transaction hash or the receipt ID.
    pub id: CryptoHash,
    /// Should be the latest field since contains unparsable by light client ExecutionStatus::Failure
    pub outcome: ExecutionOutcome,
}

impl ExecutionOutcomeWithId {
    pub fn to_hashes(&self) -> Vec<CryptoHash> {
        let mut result = vec![self.id];
        result.extend(self.outcome.to_hashes());
        result
    }
}

// /// Execution outcome with path from it to the outcome root and ID.
// #[derive(PartialEq, Clone, Default, Debug, BorshSerialize, BorshDeserialize, Eq)]
// pub struct ExecutionOutcomeWithIdAndProof {
//     pub proof: MerklePath,
//     pub block_hash: CryptoHash,
//     /// Should be the latest field since contains unparsable by light client ExecutionStatus::Failure
//     pub outcome_with_id: ExecutionOutcomeWithId,
// }

// impl ExecutionOutcomeWithIdAndProof {
//     pub fn id(&self) -> &CryptoHash {
//         &self.outcome_with_id.id
//     }
// }

pub fn verify_transaction_signature(
    transaction: &SignedTransaction,
    public_keys: &[PublicKey],
) -> bool {
    let hash = transaction.get_hash();
    let hash = hash.as_ref();
    public_keys.iter().any(|key| {
        transaction.signature.verify(hash, key)
    })
}

#[cfg(test)]
mod tests {
    use borsh::BorshDeserialize;

    use crate::crypto::{InMemorySigner, KeyType, Signature, Signer};
    use crate::serialize::to_base;

    use super::*;

    #[test]
    fn test_verify_transaction() {
        let signer = InMemorySigner::from_random("test".parse().unwrap(), KeyType::ED25519);
        let tx = Transaction {
            signer_id: "test".parse().unwrap(),
            public_key: signer.public_key(),
            nonce: 0,
            receiver_id: "test".parse().unwrap(),
            block_hash: Default::default(),
            actions: vec![],
        };
        let signature = signer.sign(&tx.get_hash_and_size().0);

        let transaction = SignedTransaction::new(signature, tx);

        let wrong_public_key = PublicKey::from_seed(KeyType::ED25519, "wrong");
        let valid_keys = vec![signer.public_key(), wrong_public_key.clone()];
        assert!(verify_transaction_signature(&transaction, &valid_keys));

        let invalid_keys = vec![wrong_public_key];
        assert!(!verify_transaction_signature(&transaction, &invalid_keys));

        let bytes = transaction.try_to_vec().unwrap();
        let decoded_tx = SignedTransaction::try_from_slice(&bytes).unwrap();
        assert!(verify_transaction_signature(&decoded_tx, &valid_keys));
    }

    /// This test is change checker for a reason - we don't expect transaction format to change.
    /// If it does - you MUST update all of the dependencies: like nearlib and other clients.
    #[test]
    fn test_serialize_transaction() {
        let public_key: PublicKey = "22skMptHjFWNyuEWY22ftn2AbLPSYpmYwGJRGwpNHbTV".parse().unwrap();
        let transaction = Transaction {
            signer_id: "test.near".parse().unwrap(),
            public_key: public_key.clone(),
            nonce: 1,
            receiver_id: "123".parse().unwrap(),
            block_hash: Default::default(),
            actions: vec![
                Action::DeployContract(DeployContractAction { code: vec![1, 2, 3] }),
                Action::FunctionCall(FunctionCallAction {
                    method_name: "qqq".to_string(),
                    args: vec![1, 2, 3],
                    gas: 1_000,
                    deposit: 1_000_000,
                }),
            ],
        };
        let signed_tx = SignedTransaction::new(Signature::empty(KeyType::ED25519), transaction);
        let new_signed_tx =
            SignedTransaction::try_from_slice(&signed_tx.try_to_vec().unwrap()).unwrap();

        assert_eq!(
            to_base(&new_signed_tx.get_hash()),
            "3iXu3iSvUdAMqDJb5HPGtk8EgVnSM7Jy6vg6ZE6JG6wZ"
        );
    }

    #[test]
    fn test_outcome_to_hashes() {
        let outcome = ExecutionOutcome {
            status: ExecutionStatus::SuccessValue(vec![123]),
            logs: vec!["123".to_string(), "321".to_string()],
            receipt_ids: vec![],
            gas_burnt: 123,
            tokens_burnt: 1234000,
            executor_id: "alice".parse().unwrap(),
            profile_data: None,
        };
        let hashes = outcome.to_hashes();
        assert_eq!(hashes.len(), 3);
    }
}
