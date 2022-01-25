use std::fmt::{self, Error, Formatter, Debug, Display};
use serde::{Serialize, Deserialize};
use crate::crypto::PublicKey;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::contract_runtime::{Balance, Nonce, Gas};
use crate::serialize::u128_dec_format;
use crate::account_id::AccountId;

#[derive(Debug, PartialEq)]
pub enum ContractPrecompilatonResult {
    ContractCompiled,
    ContractAlreadyInCache,
    CacheNotAvailable,
}

pub trait IntoVMError {
    fn into_vm_error(self) -> VMError;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum VMError {
    FunctionCallError(FunctionCallError),
    /// Serialized external error from External trait implementation.
    ExternalError(Vec<u8>),
    /// An error that is caused by an operation on an inconsistent state.
    /// E.g. an integer overflow by using a value from the given context.
    InconsistentStateError(InconsistentStateError),
    /// Error caused by caching.
    CacheError(CacheError),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FunctionCallError {
    /// Wasm compilation error
    CompilationError(CompilationError),
    /// Wasm binary env link error
    LinkError {
        msg: String,
    },
    /// Import/export resolve error
    MethodResolveError(MethodResolveError),
    /// A trap happened during execution of a binary
    WasmTrap(WasmTrap),
    WasmUnknownError {
        debug_message: String,
    },
    HostError(HostError),
    // Unused, can be reused by a future error but must be exactly one error to keep Nondeterministic
    // error borsh serialized at correct index
    _EVMError,
    /// Non-deterministic error.
    Nondeterministic(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum CacheError {
    ReadError,
    WriteError,
    DeserializationError,
    SerializationError { hash: [u8; 32] },
}

/// A kind of a trap happened during execution of a binary
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Deserialize, Serialize,
)]
pub enum WasmTrap {
    /// An `unreachable` opcode was executed.
    Unreachable,
    /// Call indirect incorrect signature trap.
    IncorrectCallIndirectSignature,
    /// Memory out of bounds trap.
    MemoryOutOfBounds,
    /// Call indirect out of bounds trap.
    CallIndirectOOB,
    /// An arithmetic exception, e.g. divided by zero.
    IllegalArithmetic,
    /// Misaligned atomic access trap.
    MisalignedAtomicAccess,
    /// Indirect call to null.
    IndirectCallToNull,
    /// Stack overflow.
    StackOverflow,
    /// Generic trap.
    GenericTrap,
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Deserialize, Serialize,
)]
pub enum MethodResolveError {
    MethodEmptyName,
    MethodNotFound,
    MethodInvalidSignature,
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Deserialize, Serialize,
)]
pub enum CompilationError {
    CodeDoesNotExist { account_id: AccountId },
    PrepareError(PrepareError),
    FloatingPointError,
    StartFunctionError,
    WasmCompileError,
    UnsupportedCompiler { msg: String },
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Deserialize, Serialize,
)]
pub enum PrepareError {
    /// Error happened while serializing the module.
    Serialization,
    /// Error happened while deserializing the module.
    Deserialization,
    /// Internal memory declaration has been found in the module.
    InternalMemoryDeclared,
    /// Gas instrumentation failed.
    ///
    /// This most likely indicates the module isn't valid.
    GasInstrumentation,
    /// Stack instrumentation failed.
    ///
    /// This  most likely indicates the module isn't valid.
    StackHeightInstrumentation,
    /// Error happened during instantiation.
    ///
    /// This might indicate that `start` function trapped, or module isn't
    /// instantiable and/or unlinkable.
    Instantiate,
    /// Error creating memory.
    Memory,
    /// Contract contains too many functions.
    TooManyFunctions,
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Deserialize, Serialize,
)]
pub enum HostError {
    /// String encoding is bad UTF-16 sequence
    BadUTF16,
    /// String encoding is bad UTF-8 sequence
    BadUTF8,
    /// Exceeded the prepaid gas
    GasExceeded,
    /// Exceeded the maximum amount of gas allowed to burn per contract
    GasLimitExceeded,
    /// Exceeded the account balance
    BalanceExceeded,
    /// Tried to call an empty method name
    EmptyMethodName,
    /// Smart contract panicked
    GuestPanic { panic_msg: String },
    /// IntegerOverflow happened during a contract execution
    IntegerOverflow,
    /// `promise_idx` does not correspond to existing promises
    InvalidPromiseIndex { promise_idx: u64 },
    /// Actions can only be appended to non-joint promise.
    CannotAppendActionToJointPromise,
    /// Returning joint promise is currently prohibited
    CannotReturnJointPromise,
    /// Accessed invalid promise result index
    InvalidPromiseResultIndex { result_idx: u64 },
    /// Accessed invalid register id
    InvalidRegisterId { register_id: u64 },
    /// Iterator `iterator_index` was invalidated after its creation by performing a mutable operation on trie
    IteratorWasInvalidated { iterator_index: u64 },
    /// Accessed memory outside the bounds
    MemoryAccessViolation,
    /// VM Logic returned an invalid receipt index
    InvalidReceiptIndex { receipt_index: u64 },
    /// Iterator index `iterator_index` does not exist
    InvalidIteratorIndex { iterator_index: u64 },
    /// VM Logic returned an invalid account id
    InvalidAccountId,
    /// VM Logic returned an invalid method name
    InvalidMethodName,
    /// VM Logic provided an invalid public key
    InvalidPublicKey,
    /// `method_name` is not allowed in view calls
    ProhibitedInView { method_name: String },
    /// The total number of logs will exceed the limit.
    NumberOfLogsExceeded { limit: u64 },
    /// The storage key length exceeded the limit.
    KeyLengthExceeded { length: u64, limit: u64 },
    /// The storage value length exceeded the limit.
    ValueLengthExceeded { length: u64, limit: u64 },
    /// The total log length exceeded the limit.
    TotalLogLengthExceeded { length: u64, limit: u64 },
    /// The maximum number of promises within a FunctionCall exceeded the limit.
    NumberPromisesExceeded { number_of_promises: u64, limit: u64 },
    /// The maximum number of input data dependencies exceeded the limit.
    NumberInputDataDependenciesExceeded { number_of_input_data_dependencies: u64, limit: u64 },
    /// The returned value length exceeded the limit.
    ReturnedValueLengthExceeded { length: u64, limit: u64 },
    /// The contract size for DeployContract action exceeded the limit.
    ContractSizeExceeded { size: u64, limit: u64 },
    /// The host function was deprecated.
    Deprecated { method_name: String },
    /// General errors for ECDSA recover.
    ECRecoverError { msg: String },

    /// work-around for Traps
    ExternalError(Vec<u8>),
    InconsistentStateError(InconsistentStateError),
    Unknown,
}

// impl wasmi::HostError for VMLogicError {}
impl wasmi::HostError for HostError {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum VMLogicError {
    /// Errors coming from native Wasm VM.
    HostError(HostError),
    /// Serialized external error from External trait implementation.
    ExternalError(Vec<u8>),
    /// An error that is caused by an operation on an inconsistent state.
    InconsistentStateError(InconsistentStateError),
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractCallError {
    MethodResolveError(MethodResolveError),
    CompilationError(CompilationError),
    ExecutionError { msg: String },
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum FunctionCallErrorSer {
    /// Wasm compilation error
    CompilationError(CompilationError),
    /// Wasm binary env link error
    LinkError {
        msg: String,
    },
    /// Import/export resolve error
    MethodResolveError(MethodResolveError),
    /// A trap happened during execution of a binary
    WasmTrap(WasmTrap),
    WasmUnknownError,
    HostError(HostError),
    // Unused, can be reused by a future error but must be exactly one error to keep ExecutionError
    // error borsh serialized at correct index
    _EVMError,
    ExecutionError(String),
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub enum ActionErrorKind {
    /// Happens when CreateAccount action tries to create an account with account_id which is already exists in the storage
    AccountAlreadyExists { account_id: AccountId },
    /// Happens when TX receiver_id doesn't exist (but action is not Action::CreateAccount)
    AccountDoesNotExist { account_id: AccountId },
    /// A top-level account ID can only be created by registrar.
    CreateAccountOnlyByRegistrar {
        account_id: AccountId,
        registrar_account_id: AccountId,
        predecessor_id: AccountId,
    },
    /// A newly created account must be under a namespace of the creator account
    CreateAccountNotAllowed { account_id: AccountId, predecessor_id: AccountId },
    /// Administrative actions like `DeployContract`, `Stake`, `AddKey`, `DeleteKey`. can be proceed only if sender=receiver
    /// or the first TX action is a `CreateAccount` action
    ActorNoPermission { account_id: AccountId, actor_id: AccountId },
    /// Account tries to remove an access key that doesn't exist
    DeleteKeyDoesNotExist { account_id: AccountId, public_key: PublicKey },
    /// The public key is already used for an existing access key
    AddKeyAlreadyExists { account_id: AccountId, public_key: PublicKey },
    /// Account is staking and can not be deleted
    DeleteAccountStaking { account_id: AccountId },
    /// ActionReceipt can't be completed, because the remaining balance will not be enough to cover storage.
    LackBalanceForState {
        /// An account which needs balance
        account_id: AccountId,
        /// Balance required to complete an action.
        #[serde(with = "u128_dec_format")]
        amount: Balance,
    },
    /// Account is not yet staked, but tries to unstake
    TriesToUnstake { account_id: AccountId },
    /// The account doesn't have enough balance to increase the stake.
    TriesToStake {
        account_id: AccountId,
        #[serde(with = "u128_dec_format")]
        stake: Balance,
        #[serde(with = "u128_dec_format")]
        locked: Balance,
        #[serde(with = "u128_dec_format")]
        balance: Balance,
    },
    InsufficientStake {
        account_id: AccountId,
        #[serde(with = "u128_dec_format")]
        stake: Balance,
        #[serde(with = "u128_dec_format")]
        minimum_stake: Balance,
    },
    /// An error occurred during a `FunctionCall` Action, parameter is debug message.
    FunctionCallError(FunctionCallErrorSer),
    /// Error occurs when a new `ActionReceipt` created by the `FunctionCall` action fails
    /// receipt validation.
    NewReceiptValidationError(ReceiptValidationError),
    /// Error occurs when a `CreateAccount` action is called on hex-characters
    /// account of length 64.  See implicit account creation NEP:
    /// <https://github.com/nearprotocol/NEPs/pull/71>.
    OnlyImplicitAccountCreationAllowed { account_id: AccountId },
    /// Delete account whose state is large is temporarily banned.
    DeleteAccountWithLargeState { account_id: AccountId },
}

/// An error happened during Acton execution
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub struct ActionError {
    /// Index of the failed action in the transaction.
    /// Action index is not defined if ActionError.kind is `ActionErrorKind::LackBalanceForState`
    pub index: Option<u64>,
    /// The kind of ActionError happened
    pub kind: ActionErrorKind,
}


/// Error returned in the ExecutionOutcome in case of failure
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub enum TxExecutionError {
    /// An error happened during Acton execution
    ActionError(ActionError),
    /// An error happened during Transaction execution
    InvalidTxError(InvalidTxError),
}

/// Describes the error for validating a receipt.
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub enum ReceiptValidationError {
    /// The `predecessor_id` of a Receipt is not valid.
    InvalidPredecessorId { account_id: String },
    /// The `receiver_id` of a Receipt is not valid.
    InvalidReceiverId { account_id: String },
    /// The `signer_id` of an ActionReceipt is not valid.
    InvalidSignerId { account_id: String },
    /// The `receiver_id` of a DataReceiver within an ActionReceipt is not valid.
    InvalidDataReceiverId { account_id: String },
    /// The length of the returned data exceeded the limit in a DataReceipt.
    ReturnedValueLengthExceeded { length: u64, limit: u64 },
    /// The number of input data dependencies exceeds the limit in an ActionReceipt.
    NumberInputDataDependenciesExceeded { number_of_input_data_dependencies: u64, limit: u64 },
    /// An error occurred while validating actions of an ActionReceipt.
    ActionsValidation(ActionsValidationError),
}

/// An error happened during TX execution
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub enum InvalidTxError {
    /// Happens if a wrong AccessKey used or AccessKey has not enough permissions
    InvalidAccessKeyError(InvalidAccessKeyError),
    /// TX signer_id is not a valid [`AccountId`]
    InvalidSignerId { signer_id: String },
    /// TX signer_id is not found in a storage
    SignerDoesNotExist { signer_id: AccountId },
    /// Transaction nonce must be `account[access_key].nonce + 1`.
    InvalidNonce { tx_nonce: Nonce, ak_nonce: Nonce },
    /// Transaction nonce is larger than the upper bound given by the block height
    NonceTooLarge { tx_nonce: Nonce, upper_bound: Nonce },
    /// TX receiver_id is not a valid AccountId
    InvalidReceiverId { receiver_id: String },
    /// TX signature is not valid
    InvalidSignature,
    /// Account does not have enough balance to cover TX cost
    NotEnoughBalance {
        signer_id: AccountId,
        #[serde(with = "u128_dec_format")]
        balance: Balance,
        #[serde(with = "u128_dec_format")]
        cost: Balance,
    },
    /// Signer account doesn't have enough balance after transaction.
    LackBalanceForState {
        /// An account which doesn't have enough balance to cover storage.
        signer_id: AccountId,
        /// Required balance to cover the state.
        #[serde(with = "u128_dec_format")]
        amount: Balance,
    },
    /// An integer overflow occurred during transaction cost estimation.
    CostOverflow,
    /// Transaction parent block hash doesn't belong to the current chain
    InvalidChain,
    /// Transaction has expired
    Expired,
    /// An error occurred while validating actions of a Transaction.
    ActionsValidation(ActionsValidationError),
    /// The size of serialized transaction exceeded the limit.
    TransactionSizeExceeded { size: u64, limit: u64 },
}

/// Describes the error for validating a list of actions.
#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub enum ActionsValidationError {
    /// The delete action must be a final aciton in transaction
    DeleteActionMustBeFinal,
    /// The total prepaid gas (for all given actions) exceeded the limit.
    TotalPrepaidGasExceeded { total_prepaid_gas: Gas, limit: Gas },
    /// The number of actions exceeded the given limit.
    TotalNumberOfActionsExceeded { total_number_of_actions: u64, limit: u64 },
    /// The total number of bytes of the method names exceeded the limit in a Add Key action.
    AddKeyMethodNamesNumberOfBytesExceeded { total_number_of_bytes: u64, limit: u64 },
    /// The length of some method name exceeded the limit in a Add Key action.
    AddKeyMethodNameLengthExceeded { length: u64, limit: u64 },
    /// Integer overflow during a compute.
    IntegerOverflow,
    /// Invalid account ID.
    InvalidAccountId { account_id: AccountId },
    /// The size of the contract code exceeded the limit in a DeployContract action.
    ContractSizeExceeded { size: u64, limit: u64 },
    /// The length of the method name exceeded the limit in a Function Call action.
    FunctionCallMethodNameLengthExceeded { length: u64, limit: u64 },
    /// The length of the arguments exceeded the limit in a Function Call action.
    FunctionCallArgumentsLengthExceeded { length: u64, limit: u64 },
    /// An attempt to stake with a public key that is not convertible to ristretto.
    UnsuitableStakingKey { public_key: PublicKey },
    /// The attached amount of gas in a FunctionCall action has to be a positive number.
    FunctionCallZeroAttachedGas,
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub enum InvalidAccessKeyError {
    /// The access key identified by the `public_key` doesn't exist for the account
    AccessKeyNotFound { account_id: AccountId, public_key: PublicKey },
    /// Transaction `receiver_id` doesn't match the access key receiver_id
    ReceiverMismatch { tx_receiver: AccountId, ak_receiver: String },
    /// Transaction method name isn't allowed by the access key
    MethodNameMismatch { method_name: String },
    /// Transaction requires a full permission access key.
    RequiresFullAccess,
    /// Access Key does not have enough allowance to cover transaction cost
    NotEnoughAllowance {
        account_id: AccountId,
        public_key: PublicKey,
        #[serde(with = "u128_dec_format")]
        allowance: Balance,
        #[serde(with = "u128_dec_format")]
        cost: Balance,
    },
    /// Having a deposit with a function call action is not allowed with a function call access key.
    DepositWithFunctionCall,
}

/// Error returned from `Runtime::apply`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// An unexpected integer overflow occurred. The likely issue is an invalid state or the transition.
    UnexpectedIntegerOverflow,
    /// An error happened during TX verification and account charging. It's likely the chunk is invalid.
    /// and should be challenged.
    InvalidTxError(InvalidTxError),
    /// Unexpected error which is typically related to the node storage corruption.
    /// It's possible the input state is invalid or malicious.
    StorageError,
    /// An error happens if `check_balance` fails, which is likely an indication of an invalid state.
    BalanceMismatchError(BalanceMismatchError),
    /// The incoming receipt didn't pass the validation, it's likely a malicious behaviour.
    ReceiptValidationError(ReceiptValidationError),
}

/// Internal
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// Key-value db internal failure
    StorageInternalError,
    /// Storage is PartialStorage and requested a missing trie node
    TrieNodeMissing,
    /// Either invalid state or key-value db is corrupted.
    /// For PartialStorage it cannot be corrupted.
    /// Error message is unreliable and for debugging purposes only. It's also probably ok to
    /// panic in every place that produces this error.
    /// We can check if db is corrupted by verifying everything in the state trie.
    StorageInconsistentState(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("{:?}", self))
    }
}

impl std::error::Error for StorageError {}

impl Display for InvalidAccessKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InvalidAccessKeyError::AccessKeyNotFound { account_id, public_key } => write!(
                f,
                "Signer {:?} doesn't have access key with the given public_key {}",
                account_id, public_key
            ),
            InvalidAccessKeyError::ReceiverMismatch { tx_receiver, ak_receiver } => write!(
                f,
                "Transaction receiver_id {:?} doesn't match the access key receiver_id {:?}",
                tx_receiver, ak_receiver
            ),
            InvalidAccessKeyError::MethodNameMismatch { method_name } => write!(
                f,
                "Transaction method name {:?} isn't allowed by the access key",
                method_name
            ),
            InvalidAccessKeyError::RequiresFullAccess => {
                write!(f, "Invalid access key type. Full-access keys are required for transactions that have multiple or non-function-call actions")
            }
            InvalidAccessKeyError::NotEnoughAllowance {
                account_id,
                public_key,
                allowance,
                cost,
            } => write!(
                f,
                "Access Key {:?}:{} does not have enough balance {} for transaction costing {}",
                account_id, public_key, allowance, cost
            ),
            InvalidAccessKeyError::DepositWithFunctionCall => {
                write!(f, "Having a deposit with a function call action is not allowed with a function call access key.")
            }
        }
    }
}

impl Display for TxExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TxExecutionError::ActionError(e) => write!(f, "{:?}", e),
            TxExecutionError::InvalidTxError(e) => write!(f, "{:?}", e),
        }
    }
}

impl From<ActionError> for TxExecutionError {
    fn from(error: ActionError) -> Self {
        TxExecutionError::ActionError(error)
    }
}

impl From<InvalidTxError> for TxExecutionError {
    fn from(error: InvalidTxError) -> Self {
        TxExecutionError::InvalidTxError(error)
    }
}

impl From<ActionErrorKind> for ActionError {
    fn from(e: ActionErrorKind) -> ActionError {
        ActionError { index: None, kind: e }
    }
}

impl Display for ReceiptValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ReceiptValidationError::InvalidPredecessorId { account_id } => {
                write!(f, "The predecessor_id `{}` of a Receipt is not valid.", account_id)
            }
            ReceiptValidationError::InvalidReceiverId { account_id } => {
                write!(f, "The receiver_id `{}` of a Receipt is not valid.", account_id)
            }
            ReceiptValidationError::InvalidSignerId { account_id } => {
                write!(f, "The signer_id `{}` of an ActionReceipt is not valid.", account_id)
            }
            ReceiptValidationError::InvalidDataReceiverId { account_id } => write!(
                f,
                "The receiver_id `{}` of a DataReceiver within an ActionReceipt is not valid.",
                account_id
            ),
            ReceiptValidationError::ReturnedValueLengthExceeded { length, limit } => write!(
                f,
                "The length of the returned data {} exceeded the limit {} in a DataReceipt",
                length, limit
            ),
            ReceiptValidationError::NumberInputDataDependenciesExceeded { number_of_input_data_dependencies, limit } => write!(
                f,
                "The number of input data dependencies {} exceeded the limit {} in an ActionReceipt",
                number_of_input_data_dependencies, limit
            ),
            ReceiptValidationError::ActionsValidation(e) => write!(f, "{}", e),
        }
    }
}

impl Display for ActionsValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ActionsValidationError::DeleteActionMustBeFinal => {
                write!(f, "The delete action must be the last action in transaction")
            }
            ActionsValidationError::TotalPrepaidGasExceeded { total_prepaid_gas, limit } => {
                write!(f, "The total prepaid gas {} exceeds the limit {}", total_prepaid_gas, limit)
            }
            ActionsValidationError::TotalNumberOfActionsExceeded {total_number_of_actions, limit } => {
                write!(
                    f,
                    "The total number of actions {} exceeds the limit {}",
                    total_number_of_actions, limit
                )
            }
            ActionsValidationError::AddKeyMethodNamesNumberOfBytesExceeded { total_number_of_bytes, limit } => write!(
                f,
                "The total number of bytes in allowed method names {} exceeds the maximum allowed number {} in a AddKey action",
                total_number_of_bytes, limit
            ),
            ActionsValidationError::AddKeyMethodNameLengthExceeded { length, limit } => write!(
                f,
                "The length of some method name {} exceeds the maximum allowed length {} in a AddKey action",
                length, limit
            ),
            ActionsValidationError::IntegerOverflow => write!(
                f,
                "Integer overflow during a compute",
            ),
            ActionsValidationError::InvalidAccountId { account_id } => write!(
                f,
                "Invalid account ID `{}`",
                account_id
            ),
            ActionsValidationError::ContractSizeExceeded { size, limit } => write!(
                f,
                "The length of the contract size {} exceeds the maximum allowed size {} in a DeployContract action",
                size, limit
            ),
            ActionsValidationError::FunctionCallMethodNameLengthExceeded { length, limit } => write!(
                f,
                "The length of the method name {} exceeds the maximum allowed length {} in a FunctionCall action",
                length, limit
            ),
            ActionsValidationError::FunctionCallArgumentsLengthExceeded { length, limit } => write!(
                f,
                "The length of the arguments {} exceeds the maximum allowed length {} in a FunctionCall action",
                length, limit
            ),
            ActionsValidationError::UnsuitableStakingKey { public_key } => write!(
                f,
                "The staking key must be ristretto compatible ED25519 key. {} is provided instead.",
                public_key,
            ),
            ActionsValidationError::FunctionCallZeroAttachedGas => write!(
                f,
                "The attached amount of gas in a FunctionCall action has to be a positive number",
            ),
        }
    }
}

impl Display for InvalidTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InvalidTxError::InvalidSignerId { signer_id } => {
                write!(f, "Invalid signer account ID {:?} according to requirements", signer_id)
            }
            InvalidTxError::SignerDoesNotExist { signer_id } => {
                write!(f, "Signer {:?} does not exist", signer_id)
            }
            InvalidTxError::InvalidAccessKeyError(access_key_error) => {
                Display::fmt(&access_key_error, f)
            }
            InvalidTxError::InvalidNonce { tx_nonce, ak_nonce } => write!(
                f,
                "Transaction nonce {} must be larger than nonce of the used access key {}",
                tx_nonce, ak_nonce
            ),
            InvalidTxError::InvalidReceiverId { receiver_id } => {
                write!(f, "Invalid receiver account ID {:?} according to requirements", receiver_id)
            }
            InvalidTxError::InvalidSignature => {
                write!(f, "Transaction is not signed with the given public key")
            }
            InvalidTxError::NotEnoughBalance { signer_id, balance, cost } => write!(
                f,
                "Sender {:?} does not have enough balance {} for operation costing {}",
                signer_id, balance, cost
            ),
            InvalidTxError::LackBalanceForState { signer_id, amount } => {
                write!(f, "Failed to execute, because the account {:?} wouldn't have enough balance to cover storage, required to have {} yoctoNEAR more", signer_id, amount)
            }
            InvalidTxError::CostOverflow => {
                write!(f, "Transaction gas or balance cost is too high")
            }
            InvalidTxError::InvalidChain => {
                write!(f, "Transaction parent block hash doesn't belong to the current chain")
            }
            InvalidTxError::Expired => {
                write!(f, "Transaction has expired")
            }
            InvalidTxError::ActionsValidation(error) => {
                write!(f, "Transaction actions validation error: {}", error)
            }
            InvalidTxError::NonceTooLarge { tx_nonce, upper_bound } => {
                write!(
                    f,
                    "Transaction nonce {} must be smaller than the access key nonce upper bound {}",
                    tx_nonce, upper_bound
                )
            }
            InvalidTxError::TransactionSizeExceeded { size, limit } => {
                write!(f, "Size of serialized transaction {} exceeded the limit {}", size, limit)
            }
        }
    }
}

impl From<InvalidAccessKeyError> for InvalidTxError {
    fn from(error: InvalidAccessKeyError) -> Self {
        InvalidTxError::InvalidAccessKeyError(error)
    }
}


impl From<ContractCallError> for FunctionCallErrorSer {
    fn from(e: ContractCallError) -> Self {
        match e {
            ContractCallError::CompilationError(e) => FunctionCallErrorSer::CompilationError(e),
            ContractCallError::MethodResolveError(e) => FunctionCallErrorSer::MethodResolveError(e),
            ContractCallError::ExecutionError { msg } => FunctionCallErrorSer::ExecutionError(msg),
        }
    }
}

impl From<FunctionCallErrorSer> for ContractCallError {
    fn from(e: FunctionCallErrorSer) -> Self {
        match e {
            FunctionCallErrorSer::CompilationError(e) => ContractCallError::CompilationError(e),
            FunctionCallErrorSer::MethodResolveError(e) => ContractCallError::MethodResolveError(e),
            FunctionCallErrorSer::ExecutionError(msg) => ContractCallError::ExecutionError { msg },
            FunctionCallErrorSer::LinkError { msg } => ContractCallError::ExecutionError { msg },
            FunctionCallErrorSer::WasmUnknownError => {
                ContractCallError::ExecutionError { msg: "unknown error".to_string() }
            }
            FunctionCallErrorSer::_EVMError => unreachable!(),
            FunctionCallErrorSer::WasmTrap(e) => {
                ContractCallError::ExecutionError { msg: format!("WASM: {:?}", e) }
            }
            FunctionCallErrorSer::HostError(e) => {
                ContractCallError::ExecutionError { msg: format!("Host: {:?}", e) }
            }
        }
    }
}

impl std::error::Error for VMLogicError {}

/// An error that is caused by an operation on an inconsistent state.
/// E.g. a deserialization error or an integer overflow.
#[derive(BorshSerialize, BorshDeserialize,Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum InconsistentStateError {
    StorageError(String),
    /// Math operation with a value from the state resulted in a integer overflow.
    IntegerOverflow,
}

impl From<HostError> for VMLogicError {
    fn from(err: HostError) -> Self {
        VMLogicError::HostError(err)
    }
}

impl From<InconsistentStateError> for VMLogicError {
    fn from(err: InconsistentStateError) -> Self {
        VMLogicError::InconsistentStateError(err)
    }
}

impl From<PrepareError> for VMError {
    fn from(err: PrepareError) -> Self {
        VMError::FunctionCallError(FunctionCallError::CompilationError(
            CompilationError::PrepareError(err),
        ))
    }
}

impl From<&VMLogicError> for VMError {
    fn from(err: &VMLogicError) -> Self {
        match err {
            VMLogicError::HostError(h) => {
                VMError::FunctionCallError(FunctionCallError::HostError(h.clone()))
            }
            VMLogicError::ExternalError(s) => VMError::ExternalError(s.clone()),
            VMLogicError::InconsistentStateError(e) => VMError::InconsistentStateError(e.clone()),
        }
    }
}

impl fmt::Display for VMLogicError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for PrepareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use PrepareError::*;
        match self {
            Serialization => write!(f, "Error happened while serializing the module."),
            Deserialization => write!(f, "Error happened while deserializing the module."),
            InternalMemoryDeclared => {
                write!(f, "Internal memory declaration has been found in the module.")
            }
            GasInstrumentation => write!(f, "Gas instrumentation failed."),
            StackHeightInstrumentation => write!(f, "Stack instrumentation failed."),
            Instantiate => write!(f, "Error happened during instantiation."),
            Memory => write!(f, "Error creating memory."),
            TooManyFunctions => write!(f, "Too many functions in contract."),
        }
    }
}

impl fmt::Display for FunctionCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            FunctionCallError::CompilationError(e) => std::fmt::Display::fmt(&e, f),
            FunctionCallError::MethodResolveError(e) => std::fmt::Display::fmt(&e, f),
            FunctionCallError::HostError(e) => std::fmt::Display::fmt(&e, f),
            FunctionCallError::LinkError { msg } => write!(f, "{}", msg),
            FunctionCallError::WasmTrap(trap) => write!(f, "WebAssembly trap: {}", trap),
            FunctionCallError::WasmUnknownError { debug_message } => {
                write!(f, "Unknown error during Wasm contract execution: {}", debug_message)
            }
            FunctionCallError::Nondeterministic(msg) => {
                write!(f, "Nondeterministic error during contract execution: {}", msg)
            }
            FunctionCallError::_EVMError => unreachable!(),
        }
    }
}

impl fmt::Display for WasmTrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            WasmTrap::Unreachable => write!(f, "An `unreachable` opcode was executed."),
            WasmTrap::IncorrectCallIndirectSignature => {
                write!(f, "Call indirect incorrect signature trap.")
            }
            WasmTrap::MemoryOutOfBounds => write!(f, "Memory out of bounds trap."),
            WasmTrap::CallIndirectOOB => write!(f, "Call indirect out of bounds trap."),
            WasmTrap::IllegalArithmetic => {
                write!(f, "An arithmetic exception, e.g. divided by zero.")
            }
            WasmTrap::MisalignedAtomicAccess => write!(f, "Misaligned atomic access trap."),
            WasmTrap::GenericTrap => write!(f, "Generic trap."),
            WasmTrap::StackOverflow => write!(f, "Stack overflow."),
            WasmTrap::IndirectCallToNull => write!(f, "Indirect call to null."),
        }
    }
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CompilationError::CodeDoesNotExist { account_id } => {
                write!(f, "cannot find contract code for account {}", account_id)
            }
            CompilationError::PrepareError(p) => write!(f, "PrepareError: {}", p),
            CompilationError::FloatingPointError => write!(f, "floating points operations not allowed in wasm"),
            CompilationError::StartFunctionError => write!(f, "start functions not allowed in wasm"),
            CompilationError::WasmCompileError => {
                write!(f, "Wasmi compilation error")
            }
            CompilationError::UnsupportedCompiler { msg } => {
                write!(f, "Unsupported compiler: {}", msg)
            }
        }
    }
}

impl fmt::Display for MethodResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            VMError::FunctionCallError(err) => fmt::Display::fmt(err, f),
            VMError::ExternalError(_err) => write!(f, "Serialized ExternalError"),
            VMError::InconsistentStateError(err) => fmt::Display::fmt(err, f),
            VMError::CacheError(err) => write!(f, "Cache error: {:?}", err),
        }
    }
}

impl std::fmt::Display for InconsistentStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InconsistentStateError::StorageError(err) => write!(f, "Storage error: {:?}", err),
            InconsistentStateError::IntegerOverflow => write!(
                f,
                "Math operation with a value from the state resulted in a integer overflow.",
            ),
        }
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use HostError::*;
        match self {
            BadUTF8 => write!(f, "String encoding is bad UTF-8 sequence."),
            BadUTF16 => write!(f, "String encoding is bad UTF-16 sequence."),
            GasExceeded => write!(f, "Exceeded the prepaid gas."),
            GasLimitExceeded => write!(f, "Exceeded the maximum amount of gas allowed to burn per contract."),
            BalanceExceeded => write!(f, "Exceeded the account balance."),
            EmptyMethodName => write!(f, "Tried to call an empty method name."),
            GuestPanic { panic_msg } => write!(f, "Smart contract panicked: {}", panic_msg),
            IntegerOverflow => write!(f, "Integer overflow."),
            InvalidIteratorIndex { iterator_index } => write!(f, "Iterator index {:?} does not exist", iterator_index),
            InvalidPromiseIndex { promise_idx } => write!(f, "{:?} does not correspond to existing promises", promise_idx),
            CannotAppendActionToJointPromise => write!(f, "Actions can only be appended to non-joint promise."),
            CannotReturnJointPromise => write!(f, "Returning joint promise is currently prohibited."),
            InvalidPromiseResultIndex { result_idx } => write!(f, "Accessed invalid promise result index: {:?}", result_idx),
            InvalidRegisterId { register_id } => write!(f, "Accessed invalid register id: {:?}", register_id),
            IteratorWasInvalidated { iterator_index } => write!(f, "Iterator {:?} was invalidated after its creation by performing a mutable operation on trie", iterator_index),
            MemoryAccessViolation => write!(f, "Accessed memory outside the bounds."),
            InvalidReceiptIndex { receipt_index } => write!(f, "VM Logic returned an invalid receipt index: {:?}", receipt_index),
            InvalidAccountId => write!(f, "VM Logic returned an invalid account id"),
            InvalidMethodName => write!(f, "VM Logic returned an invalid method name"),
            InvalidPublicKey => write!(f, "VM Logic provided an invalid public key"),
            ProhibitedInView { method_name } => write!(f, "{} is not allowed in view calls", method_name),
            NumberOfLogsExceeded { limit } => write!(f, "The number of logs will exceed the limit {}", limit),
            KeyLengthExceeded { length, limit } => write!(f, "The length of a storage key {} exceeds the limit {}", length, limit),
            ValueLengthExceeded { length, limit } => write!(f, "The length of a storage value {} exceeds the limit {}", length, limit),
            TotalLogLengthExceeded{ length, limit } => write!(f, "The length of a log message {} exceeds the limit {}", length, limit),
            NumberPromisesExceeded { number_of_promises, limit } => write!(f, "The number of promises within a FunctionCall {} exceeds the limit {}", number_of_promises, limit),
            NumberInputDataDependenciesExceeded { number_of_input_data_dependencies, limit } => write!(f, "The number of input data dependencies {} exceeds the limit {}", number_of_input_data_dependencies, limit),
            ReturnedValueLengthExceeded { length, limit } => write!(f, "The length of a returned value {} exceeds the limit {}", length, limit),
            ContractSizeExceeded { size, limit } => write!(f, "The size of a contract code in DeployContract action {} exceeds the limit {}", size, limit),
            Deprecated {method_name}=> write!(f, "Attempted to call deprecated host function {}", method_name),
            ECRecoverError { msg } => write!(f, "ECDSA recover error: {}", msg),
            ExternalError(_) => write!(f, "external error"),
            InconsistentStateError(e) => write!(f, "InconsistentStateError: {}", e),
            Unknown => write!(f, "unkonw error"),
        }
    }
}

impl From<StorageError> for RuntimeError {
    fn from(_: StorageError) -> Self {
        RuntimeError::StorageError
    }
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct IntegerOverflowError;

impl From<IntegerOverflowError> for InvalidTxError {
    fn from(_: IntegerOverflowError) -> Self {
        InvalidTxError::CostOverflow
    }
}

impl From<BalanceMismatchError> for RuntimeError {
    fn from(e: BalanceMismatchError) -> Self {
        RuntimeError::BalanceMismatchError(e)
    }
}

impl From<IntegerOverflowError> for RuntimeError {
    fn from(_: IntegerOverflowError) -> Self {
        RuntimeError::UnexpectedIntegerOverflow
    }
}

impl From<InvalidTxError> for RuntimeError {
    fn from(e: InvalidTxError) -> Self {
        RuntimeError::InvalidTxError(e)
    }
}

// /// Type-erased error used to shuttle some concrete error coming from `External`
// /// through vm-logic.
// ///
// /// The caller is supposed to downcast this to a concrete error type they should
// /// know. This would be just `Box<dyn Any + Eq>` if the latter actually worked.
// pub struct AnyError {
//     any: Box<dyn AnyEq>,
// }

// impl AnyError {
//     pub fn new<E: Any + Eq + Send + Sync + 'static>(err: E) -> AnyError {
//         AnyError { any: Box::new(err) }
//     }
//     pub fn downcast<E: Any + Eq + Send + Sync + 'static>(self) -> Result<E, ()> {
//         match self.any.into_any().downcast::<E>() {
//             Ok(it) => Ok(*it),
//             Err(_) => Err(()),
//         }
//     }
// }

// impl PartialEq for AnyError {
//     fn eq(&self, other: &Self) -> bool {
//         self.any.any_eq(&*other.any)
//     }
// }

// impl Eq for AnyError {}

// impl fmt::Debug for AnyError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::Debug::fmt(self.any.as_any(), f)
//     }
// }

// trait AnyEq: Any + Send + Sync {
//     fn any_eq(&self, rhs: &dyn AnyEq) -> bool;
//     fn as_any(&self) -> &dyn Any;
//     fn into_any(self: Box<Self>) -> Box<dyn Any>;
// }

// impl<T: Any + Eq + Sized + Send + Sync> AnyEq for T {
//     fn any_eq(&self, rhs: &dyn AnyEq) -> bool {
//         match rhs.as_any().downcast_ref::<Self>() {
//             Some(rhs) => self == rhs,
//             None => false,
//         }
//     }
//     fn as_any(&self) -> &dyn Any {
//         &*self
//     }
//     fn into_any(self: Box<Self>) -> Box<dyn Any> {
//         self
//     }
// }

/// Happens when the input balance doesn't match the output balance in Runtime apply.
#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Deserialize, Serialize,
)]
pub struct BalanceMismatchError {
    // Input balances
    #[serde(with = "u128_dec_format")]
    pub initial_accounts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub incoming_receipts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub processed_delayed_receipts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub initial_postponed_receipts_balance: Balance,
    // Output balances
    #[serde(with = "u128_dec_format")]
    pub final_accounts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub outgoing_receipts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub new_delayed_receipts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub final_postponed_receipts_balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub tx_burnt_amount: Balance,
    #[serde(with = "u128_dec_format")]
    pub slashed_burnt_amount: Balance,
    #[serde(with = "u128_dec_format")]
    pub other_burnt_amount: Balance,
}

impl Display for BalanceMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // Using saturating add to avoid overflow in display
        let initial_balance = 0u128
            .saturating_add(self.initial_accounts_balance)
            .saturating_add(self.incoming_receipts_balance)
            .saturating_add(self.processed_delayed_receipts_balance)
            .saturating_add(self.initial_postponed_receipts_balance);
        let final_balance = self
            .final_accounts_balance
            .saturating_add(self.outgoing_receipts_balance)
            .saturating_add(self.new_delayed_receipts_balance)
            .saturating_add(self.final_postponed_receipts_balance)
            .saturating_add(self.tx_burnt_amount)
            .saturating_add(self.slashed_burnt_amount)
            .saturating_add(self.other_burnt_amount);
        write!(
            f,
            "Balance Mismatch Error. The input balance {} doesn't match output balance {}\n\
             Inputs:\n\
             \tInitial accounts balance sum: {}\n\
             \tIncoming receipts balance sum: {}\n\
             \tProcessed delayed receipts balance sum: {}\n\
             \tInitial postponed receipts balance sum: {}\n\
             Outputs:\n\
             \tFinal accounts balance sum: {}\n\
             \tOutgoing receipts balance sum: {}\n\
             \tNew delayed receipts balance sum: {}\n\
             \tFinal postponed receipts balance sum: {}\n\
             \tTx fees burnt amount: {}\n\
             \tSlashed amount: {}\n\
             \tOther burnt amount: {}",
            initial_balance,
            final_balance,
            self.initial_accounts_balance,
            self.incoming_receipts_balance,
            self.processed_delayed_receipts_balance,
            self.initial_postponed_receipts_balance,
            self.final_accounts_balance,
            self.outgoing_receipts_balance,
            self.new_delayed_receipts_balance,
            self.final_postponed_receipts_balance,
            self.tx_burnt_amount,
            self.slashed_burnt_amount,
            self.other_burnt_amount,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CompilationError, FunctionCallError, MethodResolveError, PrepareError, VMError};

    #[test]
    fn test_display() {
        // TODO: proper printing
        assert_eq!(
            VMError::FunctionCallError(FunctionCallError::MethodResolveError(
                MethodResolveError::MethodInvalidSignature
            ))
            .to_string(),
            "MethodInvalidSignature"
        );
        assert_eq!(
            VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::StackHeightInstrumentation)
            ))
            .to_string(),
            "PrepareError: Stack instrumentation failed."
        );
    }
}

