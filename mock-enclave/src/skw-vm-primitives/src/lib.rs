pub mod errors;

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

    pub fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
        sha2::Sha256::digest(bytes).into()
    }

}

pub mod fees {
    use num_rational::Rational;
    use crate::contract_runtime::Gas;

    /// Costs associated with an object that can only be sent over the network (and executed
    /// by the receiver).
    /// NOTE: `send_sir` or `send_not_sir` fees are usually burned when the item is being created.
    /// And `execution` fee is burned when the item is being executed.
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Fee {
        /// Fee for sending an object from the sender to itself, guaranteeing that it does not leave
        /// the shard.
        pub send_sir: Gas,
        /// Fee for sending an object potentially across the shards.
        pub send_not_sir: Gas,
        /// Fee for executing the object.
        pub execution: Gas,
    }

    impl Fee {
        #[inline]
        pub fn send_fee(&self, sir: bool) -> Gas {
            if sir {
                self.send_sir
            } else {
                self.send_not_sir
            }
        }

        pub fn exec_fee(&self) -> Gas {
            self.execution
        }

        /// The minimum fee to send and execute.
        fn min_send_and_exec_fee(&self) -> Gas {
            std::cmp::min(self.send_sir, self.send_not_sir) + self.execution
        }
    }


    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct RuntimeFeesConfig {
        /// Describes the cost of creating an action receipt, `ActionReceipt`, excluding the actual cost
        /// of actions.
        /// - `send` cost is burned when a receipt is created using `promise_create` or
        ///     `promise_batch_create`
        /// - `exec` cost is burned when the receipt is being executed.
        pub action_receipt_creation_config: Fee,
        /// Describes the cost of creating a data receipt, `DataReceipt`.
        pub data_receipt_creation_config: DataReceiptCreationConfig,
        /// Describes the cost of creating a certain action, `Action`. Includes all variants.
        pub action_creation_config: ActionCreationConfig,
        /// Describes fees for storage.
        pub storage_usage_config: StorageUsageConfig,

        /// Fraction of the burnt gas to reward to the contract account for execution.
        pub burnt_gas_reward: Rational,

        /// Pessimistic gas price inflation ratio.
        pub pessimistic_gas_price_inflation_ratio: Rational,
    }


    /// Describes the cost of creating a data receipt, `DataReceipt`.
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct DataReceiptCreationConfig {
        /// Base cost of creating a data receipt.
        /// Both `send` and `exec` costs are burned when a new receipt has input dependencies. The gas
        /// is charged for each input dependency. The dependencies are specified when a receipt is
        /// created using `promise_then` and `promise_batch_then`.
        /// NOTE: Any receipt with output dependencies will produce data receipts. Even if it fails.
        /// Even if the last action is not a function call (in case of success it will return empty
        /// value).
        pub base_cost: Fee,
        /// Additional cost per byte sent.
        /// Both `send` and `exec` costs are burned when a function call finishes execution and returns
        /// `N` bytes of data to every output dependency. For each output dependency the cost is
        /// `(send(sir) + exec()) * N`.
        pub cost_per_byte: Fee,
    }

    /// Describes the cost of creating a specific action, `Action`. Includes all variants.
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct ActionCreationConfig {
        /// Base cost of creating an account.
        pub create_account_cost: Fee,

        /// Base cost of deploying a contract.
        pub deploy_contract_cost: Fee,
        /// Cost per byte of deploying a contract.
        pub deploy_contract_cost_per_byte: Fee,

        /// Base cost of calling a function.
        pub function_call_cost: Fee,
        /// Cost per byte of method name and arguments of calling a function.
        pub function_call_cost_per_byte: Fee,

        /// Base cost of making a transfer.
        pub transfer_cost: Fee,

        /// Base cost of deleting an account.
        pub delete_account_cost: Fee,
    }

    /// Describes cost of storage per block
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct StorageUsageConfig {
        /// Number of bytes for an account record, including rounding up for account id.
        pub num_bytes_account: u64,
        /// Additional number of bytes for a k/v record
        pub num_extra_bytes_record: u64,
    }

    impl RuntimeFeesConfig {
        pub fn test() -> Self {
            #[allow(clippy::unreadable_literal)]
            Self {
                action_receipt_creation_config: Fee {
                    send_sir: 108059500000,
                    send_not_sir: 108059500000,
                    execution: 108059500000,
                },
                data_receipt_creation_config: DataReceiptCreationConfig {
                    base_cost: Fee {
                        send_sir: 4697339419375,
                        send_not_sir: 4697339419375,
                        execution: 4697339419375,
                    },
                    cost_per_byte: Fee {
                        send_sir: 59357464,
                        send_not_sir: 59357464,
                        execution: 59357464,
                    },
                },
                action_creation_config: ActionCreationConfig {
                    create_account_cost: Fee {
                        send_sir: 99607375000,
                        send_not_sir: 99607375000,
                        execution: 99607375000,
                    },
                    deploy_contract_cost: Fee {
                        send_sir: 184765750000,
                        send_not_sir: 184765750000,
                        execution: 184765750000,
                    },
                    deploy_contract_cost_per_byte: Fee {
                        send_sir: 6812999,
                        send_not_sir: 6812999,
                        execution: 6812999,
                    },
                    function_call_cost: Fee {
                        send_sir: 2319861500000,
                        send_not_sir: 2319861500000,
                        execution: 2319861500000,
                    },
                    function_call_cost_per_byte: Fee {
                        send_sir: 2235934,
                        send_not_sir: 2235934,
                        execution: 2235934,
                    },
                    transfer_cost: Fee {
                        send_sir: 115123062500,
                        send_not_sir: 115123062500,
                        execution: 115123062500,
                    },
                    delete_account_cost: Fee {
                        send_sir: 147489000000,
                        send_not_sir: 147489000000,
                        execution: 147489000000,
                    },
                },
                storage_usage_config: StorageUsageConfig {
                    // See Account in core/primitives/src/account.rs for the data structure.
                    // TODO(2291): figure out value for the mainnet.
                    num_bytes_account: 100,
                    num_extra_bytes_record: 40,
                },
                burnt_gas_reward: Rational::new(3, 10),
                pessimistic_gas_price_inflation_ratio: Rational::new(103, 100),
            }
        }

        pub fn free() -> Self {
            let free = Fee { send_sir: 0, send_not_sir: 0, execution: 0 };
            RuntimeFeesConfig {
                action_receipt_creation_config: free.clone(),
                data_receipt_creation_config: DataReceiptCreationConfig {
                    base_cost: free.clone(),
                    cost_per_byte: free.clone(),
                },
                action_creation_config: ActionCreationConfig {
                    create_account_cost: free.clone(),
                    deploy_contract_cost: free.clone(),
                    deploy_contract_cost_per_byte: free.clone(),
                    function_call_cost: free.clone(),
                    function_call_cost_per_byte: free.clone(),
                    transfer_cost: free.clone(),
                    delete_account_cost: free,
                },
                storage_usage_config: StorageUsageConfig {
                    num_bytes_account: 0,
                    num_extra_bytes_record: 0,
                },
                burnt_gas_reward: Rational::from_integer(0),
                pessimistic_gas_price_inflation_ratio: Rational::from_integer(0),
            }
        }

        /// The minimum amount of gas required to create and execute a new receipt with a function call
        /// action.
        /// This amount is used to determine how many receipts can be created, send and executed for
        /// some amount of prepaid gas using function calls.
        pub fn min_receipt_with_function_call_gas(&self) -> Gas {
            self.action_receipt_creation_config.min_send_and_exec_fee()
                + self.action_creation_config.function_call_cost.min_send_and_exec_fee()
        }
    }

    /// Helper functions for computing Transfer fees.
    /// In case of implicit account creation they always include extra fees for the CreateAccount and
    /// AddFullAccessKey actions that are implicit.
    /// We can assume that no overflow will happen here.
    pub fn transfer_exec_fee(cfg: &ActionCreationConfig, is_receiver_implicit: bool) -> Gas {
        if is_receiver_implicit {
            cfg.create_account_cost.exec_fee()
                + cfg.transfer_cost.exec_fee()
        } else {
            cfg.transfer_cost.exec_fee()
        }
    }

    pub fn transfer_send_fee(
        cfg: &ActionCreationConfig,
        sender_is_receiver: bool,
        is_receiver_implicit: bool,
    ) -> Gas {
        if is_receiver_implicit {
            cfg.create_account_cost.send_fee(sender_is_receiver)
                + cfg.transfer_cost.send_fee(sender_is_receiver)
        } else {
            cfg.transfer_cost.send_fee(sender_is_receiver)
        }
    }
}

pub mod account_id {
    use crate::errors::{ParseAccountError, ParseErrorKind};
    use std::{fmt, str::FromStr};
    use serde::{de, ser};

    #[derive(Eq, Ord, Hash, Clone, Debug, PartialEq, PartialOrd)]
    pub struct AccountId(Box<str>);

    impl AccountId {
        pub const MIN_LEN: usize = 2;
        pub const MAX_LEN: usize = 64;

        pub fn as_str(&self) -> &str {
            self
        }

        pub fn is_top_level(&self) -> bool {
            !self.is_system() && !self.contains('.')
        }

        pub fn is_sub_account_of(&self, parent: &AccountId) -> bool {
            self.strip_suffix(parent.as_str())
                .map_or(false, |s| !s.is_empty() && s.find('.') == Some(s.len() - 1))
        }

        pub fn is_implicit(&self) -> bool {
            self.len() == 64 && self.as_bytes().iter().all(|b| matches!(b, b'a'..=b'f' | b'0'..=b'9'))
        }

        pub fn is_system(&self) -> bool {
            self.as_str() == "system"
        }

        pub fn validate(account_id: &str) -> Result<(), ParseAccountError> {
            if account_id.len() < AccountId::MIN_LEN {
                Err(ParseAccountError { kind: ParseErrorKind::TooShort, char: None })
            } else if account_id.len() > AccountId::MAX_LEN {
                Err(ParseAccountError { kind: ParseErrorKind::TooLong, char: None })
            } else {
                // Adapted from https://github.com/near/near-sdk-rs/blob/fd7d4f82d0dfd15f824a1cf110e552e940ea9073/near-sdk/src/environment/env.rs#L819

                // NOTE: We don't want to use Regex here, because it requires extra time to compile it.
                // The valid account ID regex is /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/
                // Instead the implementation is based on the previous character checks.

                // We can safely assume that last char was a separator.
                let mut last_char_is_separator = true;

                let mut this = None;
                for (i, c) in account_id.chars().enumerate() {
                    this.replace((i, c));
                    let current_char_is_separator = match c {
                        'a'..='z' | '0'..='9' => false,
                        '-' | '_' | '.' => true,
                        _ => {
                            return Err(ParseAccountError {
                                kind: ParseErrorKind::InvalidChar,
                                char: this,
                            });
                        }
                    };
                    if current_char_is_separator && last_char_is_separator {
                        return Err(ParseAccountError {
                            kind: ParseErrorKind::RedundantSeparator,
                            char: this,
                        });
                    }
                    last_char_is_separator = current_char_is_separator;
                }

                if last_char_is_separator {
                    return Err(ParseAccountError {
                        kind: ParseErrorKind::RedundantSeparator,
                        char: this,
                    });
                }
                Ok(())
            }
        }

        pub fn new_unvalidated(account_id: String) -> Self {
            Self(account_id.into_boxed_str())
        }
    }

    impl std::ops::Deref for AccountId {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.0.as_ref()
        }
    }

    impl AsRef<str> for AccountId {
        fn as_ref(&self) -> &str {
            self
        }
    }

    impl std::borrow::Borrow<str> for AccountId {
        fn borrow(&self) -> &str {
            self
        }
    }

    impl FromStr for AccountId {
        type Err = ParseAccountError;

        fn from_str(account_id: &str) -> Result<Self, Self::Err> {
            Self::validate(account_id)?;
            Ok(Self(account_id.into()))
        }
    }

    impl TryFrom<Box<str>> for AccountId {
        type Error = ParseAccountError;

        fn try_from(account_id: Box<str>) -> Result<Self, Self::Error> {
            Self::validate(&account_id)?;
            Ok(Self(account_id))
        }
    }

    impl TryFrom<String> for AccountId {
        type Error = ParseAccountError;

        fn try_from(account_id: String) -> Result<Self, Self::Error> {
            Self::validate(&account_id)?;
            Ok(Self(account_id.into_boxed_str()))
        }
    }

    impl fmt::Display for AccountId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(&self.0, f)
        }
    }

    impl From<AccountId> for String {
        fn from(account_id: AccountId) -> Self {
            account_id.0.into_string()
        }
    }

    impl From<AccountId> for Box<str> {
        fn from(value: AccountId) -> Box<str> {
            value.0
        }
    }

    impl ser::Serialize for AccountId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    impl<'de> de::Deserialize<'de> for AccountId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let account_id = Box::<str>::deserialize(deserializer)?;
            AccountId::validate(&account_id).map_err(|err| {
                de::Error::custom(format!("invalid value: \"{}\", {}", account_id, err))
            })?;
            Ok(AccountId(account_id))
        }
    }

}

pub mod config {
    use crate::contract_runtime::Gas;
    use std::fmt;
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    use serde::{Deserialize, Serialize};
    
    #[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
    pub struct VMConfig {
        /// Costs for runtime externals
        pub ext_costs: ExtCostsConfig,

        /// Gas cost of a growing memory by single page.
        pub grow_mem_cost: u32,
        /// Gas cost of a regular operation.
        pub regular_op_cost: u32,

        /// Describes limits for VM and Runtime.
        pub limit_config: VMLimitConfig,
    }

    /// Describes limits for VM and Runtime.
    /// TODO #4139: consider switching to strongly-typed wrappers instead of raw quantities
    #[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
    pub struct VMLimitConfig {
        /// Max amount of gas that can be used, excluding gas attached to promises.
        pub max_gas_burnt: Gas,

        /// How tall the stack is allowed to grow?
        ///
        /// See <https://wiki.parity.io/WebAssembly-StackHeight> to find out
        /// how the stack frame cost is calculated.
        pub max_stack_height: u32,
        /// The initial number of memory pages.
        /// NOTE: It's not a limiter itself, but it's a value we use for initial_memory_pages.
        pub initial_memory_pages: u32,
        /// What is the maximal memory pages amount is allowed to have for
        /// a contract.
        pub max_memory_pages: u32,

        /// Limit of memory used by registers.
        pub registers_memory_limit: u64,
        /// Maximum number of bytes that can be stored in a single register.
        pub max_register_size: u64,
        /// Maximum number of registers that can be used simultaneously.
        pub max_number_registers: u64,

        /// Maximum number of log entries.
        pub max_number_logs: u64,
        /// Maximum total length in bytes of all log messages.
        pub max_total_log_length: u64,

        /// Max total prepaid gas for all function call actions per receipt.
        pub max_total_prepaid_gas: Gas,

        /// Max number of actions per receipt.
        pub max_actions_per_receipt: u64,
        /// Max total length of all method names (including terminating character) for a function call
        /// permission access key.
        pub max_number_bytes_method_names: u64,
        /// Max length of any method name (without terminating character).
        pub max_length_method_name: u64,
        /// Max length of arguments in a function call action.
        pub max_arguments_length: u64,
        /// Max length of returned data
        pub max_length_returned_data: u64,
        /// Max contract size
        pub max_contract_size: u64,
        /// Max transaction size
        pub max_transaction_size: u64,
        /// Max storage key size
        pub max_length_storage_key: u64,
        /// Max storage value size
        pub max_length_storage_value: u64,
        /// Max number of promises that a function call can create
        pub max_promises_per_function_call_action: u64,
        /// Max number of input data dependencies
        pub max_number_input_data_dependencies: u64,
        /// If present, stores max number of functions in one contract
        pub max_functions_number_per_contract: Option<u64>,
    }

    impl VMConfig {
        pub fn test() -> VMConfig {
            VMConfig {
                ext_costs: ExtCostsConfig::test(),
                grow_mem_cost: 1,
                regular_op_cost: (SAFETY_MULTIPLIER as u32) * 1285457,
                limit_config: VMLimitConfig::test(),
            }
        }

        /// Computes non-cryptographically-proof hash. The computation is fast but not cryptographically
        /// secure.
        pub fn non_crypto_hash(&self) -> u64 {
            let mut s = DefaultHasher::new();
            self.hash(&mut s);
            s.finish()
        }

        pub fn free() -> Self {
            Self {
                ext_costs: ExtCostsConfig::free(),
                grow_mem_cost: 0,
                regular_op_cost: 0,
                // We shouldn't have any costs in the limit config.
                limit_config: VMLimitConfig { max_gas_burnt: u64::MAX, ..VMLimitConfig::test() },
            }
        }
    }

    impl VMLimitConfig {
        pub fn test() -> Self {
            Self {
                max_gas_burnt: 2 * 10u64.pow(14), // with 10**15 block gas limit this will allow 5 calls.

                // NOTE: Stack height has to be 16K, otherwise Wasmer produces non-deterministic results.
                // For experimentation try `test_stack_overflow`.
                max_stack_height: 16 * 1024, // 16Kib of stack.
                initial_memory_pages: 2u32.pow(10), // 64Mib of memory.
                max_memory_pages: 2u32.pow(11),     // 128Mib of memory.

                // By default registers are limited by 1GiB of memory.
                registers_memory_limit: 2u64.pow(30),
                // By default each register is limited by 100MiB of memory.
                max_register_size: 2u64.pow(20) * 100,
                // By default there is at most 100 registers.
                max_number_registers: 100,

                max_number_logs: 100,
                // Total logs size is 16Kib
                max_total_log_length: 16 * 1024,

                // Updating the maximum prepaid gas to limit the maximum depth of a transaction to 64
                // blocks.
                // This based on `63 * min_receipt_with_function_call_gas()`. Where 63 is max depth - 1.
                max_total_prepaid_gas: 300 * 10u64.pow(12),

                // Safety limit. Unlikely to hit it for most common transactions and receipts.
                max_actions_per_receipt: 100,
                // Should be low enough to deserialize an access key without paying.
                max_number_bytes_method_names: 2000,
                max_length_method_name: 256,            // basic safety limit
                max_arguments_length: 4 * 2u64.pow(20), // 4 Mib
                max_length_returned_data: 4 * 2u64.pow(20), // 4 Mib
                max_contract_size: 4 * 2u64.pow(20),    // 4 Mib,
                max_transaction_size: 4 * 2u64.pow(20), // 4 Mib

                max_length_storage_key: 4 * 2u64.pow(20), // 4 Mib
                max_length_storage_value: 4 * 2u64.pow(20), // 4 Mib
                // Safety limit and unlikely abusable.
                max_promises_per_function_call_action: 1024,
                // Unlikely to hit it for normal development.
                max_number_input_data_dependencies: 128,
                max_functions_number_per_contract: None,
            }
        }
    }

    /// Configuration of view methods execution, during which no costs should be charged.
    #[derive(Default, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
    pub struct ViewConfig {
        /// If specified, defines max burnt gas per view method.
        pub max_gas_burnt: Gas,
    }

    // Type of an action, used in fees logic.
    #[derive(Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug, PartialOrd, Ord)]
    #[allow(non_camel_case_types)]
    pub enum ActionCosts {
        create_account,
        delete_account,
        deploy_contract,
        function_call,
        transfer,
        value_return,
        new_receipt,

        // NOTE: this should be the last element of the enum.
        __count,
    }


    impl fmt::Display for ActionCosts {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", ActionCosts::name_of(*self as usize))
        }
    }

    impl ActionCosts {
        pub const fn count() -> usize {
            ActionCosts::__count as usize
        }

        pub fn name_of(index: usize) -> &'static str {
            vec![
                "create_account",
                "delete_account",
                "deploy_contract",
                "function_call",
                "transfer",
                "value_return",
                "new_receipt",
            ][index]
        }
    }

    impl fmt::Display for ExtCosts {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", ExtCosts::name_of(*self as usize))
        }
    }


    #[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ExtCostsConfig {
        /// Base cost for calling a host function.
        pub base: Gas,

        /// Base cost of loading and compiling contract
        pub contract_compile_base: Gas,
        /// Cost of the execution to load and compile contract
        pub contract_compile_bytes: Gas,

        /// Base cost for guest memory read
        pub read_memory_base: Gas,
        /// Cost for guest memory read
        pub read_memory_byte: Gas,

        /// Base cost for guest memory write
        pub write_memory_base: Gas,
        /// Cost for guest memory write per byte
        pub write_memory_byte: Gas,

        /// Base cost for reading from register
        pub read_register_base: Gas,
        /// Cost for reading byte from register
        pub read_register_byte: Gas,

        /// Base cost for writing into register
        pub write_register_base: Gas,
        /// Cost for writing byte into register
        pub write_register_byte: Gas,

        /// Base cost of decoding utf8. It's used for `log_utf8` and `panic_utf8`.
        pub utf8_decoding_base: Gas,
        /// Cost per byte of decoding utf8. It's used for `log_utf8` and `panic_utf8`.
        pub utf8_decoding_byte: Gas,

        /// Base cost of decoding utf16. It's used for `log_utf16`.
        pub utf16_decoding_base: Gas,
        /// Cost per byte of decoding utf16. It's used for `log_utf16`.
        pub utf16_decoding_byte: Gas,

        /// Cost of getting sha256 base
        pub sha256_base: Gas,
        /// Cost of getting sha256 per byte
        pub sha256_byte: Gas,

        /// Cost of getting sha256 base
        pub keccak256_base: Gas,
        /// Cost of getting sha256 per byte
        pub keccak256_byte: Gas,

        /// Cost of getting sha256 base
        pub keccak512_base: Gas,
        /// Cost of getting sha256 per byte
        pub keccak512_byte: Gas,

        /// Cost of getting ripemd160 base
        pub ripemd160_base: Gas,
        /// Cost of getting ripemd160 per message block
        pub ripemd160_block: Gas,

        /// Cost of calling ecrecover
        pub ecrecover_base: Gas,

        /// Cost for calling logging.
        pub log_base: Gas,
        /// Cost for logging per byte
        pub log_byte: Gas,

        // ###############
        // # Storage API #
        // ###############
        /// Storage trie write key base cost
        pub storage_write_base: Gas,
        /// Storage trie write key per byte cost
        pub storage_write_key_byte: Gas,
        /// Storage trie write value per byte cost
        pub storage_write_value_byte: Gas,
        /// Storage trie write cost per byte of evicted value.
        pub storage_write_evicted_byte: Gas,

        /// Storage trie read key base cost
        pub storage_read_base: Gas,
        /// Storage trie read key per byte cost
        pub storage_read_key_byte: Gas,
        /// Storage trie read value cost per byte cost
        pub storage_read_value_byte: Gas,

        /// Remove key from trie base cost
        pub storage_remove_base: Gas,
        /// Remove key from trie per byte cost
        pub storage_remove_key_byte: Gas,
        /// Remove key from trie ret value byte cost
        pub storage_remove_ret_value_byte: Gas,

        /// Storage trie check for key existence cost base
        pub storage_has_key_base: Gas,
        /// Storage trie check for key existence per key byte
        pub storage_has_key_byte: Gas,

        /// Cost per touched trie node
        pub touching_trie_node: Gas,

        // ###############
        // # Promise API #
        // ###############
        /// Cost for calling `promise_and`
        pub promise_and_base: Gas,
        /// Cost for calling `promise_and` for each promise
        pub promise_and_per_promise: Gas,
        /// Cost for calling `promise_return`
        pub promise_return: Gas,
    }

    // We multiply the actual computed costs by the fixed factor to ensure we
    // have certain reserve for further gas price variation.
    const SAFETY_MULTIPLIER: u64 = 3;

    impl ExtCostsConfig {
        pub fn test() -> ExtCostsConfig {
            ExtCostsConfig {
                base: SAFETY_MULTIPLIER * 88256037,
                contract_compile_base: SAFETY_MULTIPLIER * 11815321,
                contract_compile_bytes: SAFETY_MULTIPLIER * 72250,
                read_memory_base: SAFETY_MULTIPLIER * 869954400,
                read_memory_byte: SAFETY_MULTIPLIER * 1267111,
                write_memory_base: SAFETY_MULTIPLIER * 934598287,
                write_memory_byte: SAFETY_MULTIPLIER * 907924,
                read_register_base: SAFETY_MULTIPLIER * 839055062,
                read_register_byte: SAFETY_MULTIPLIER * 32854,
                write_register_base: SAFETY_MULTIPLIER * 955174162,
                write_register_byte: SAFETY_MULTIPLIER * 1267188,
                utf8_decoding_base: SAFETY_MULTIPLIER * 1037259687,
                utf8_decoding_byte: SAFETY_MULTIPLIER * 97193493,
                utf16_decoding_base: SAFETY_MULTIPLIER * 1181104350,
                utf16_decoding_byte: SAFETY_MULTIPLIER * 54525831,
                sha256_base: SAFETY_MULTIPLIER * 1513656750,
                sha256_byte: SAFETY_MULTIPLIER * 8039117,
                keccak256_base: SAFETY_MULTIPLIER * 1959830425,
                keccak256_byte: SAFETY_MULTIPLIER * 7157035,
                keccak512_base: SAFETY_MULTIPLIER * 1937129412,
                keccak512_byte: SAFETY_MULTIPLIER * 12216567,
                ripemd160_base: SAFETY_MULTIPLIER * 284558362,
                // Cost per byte is 3542227. There are 64 bytes in a block.
                ripemd160_block: SAFETY_MULTIPLIER * 226702528,
                ecrecover_base: SAFETY_MULTIPLIER * 1121789875000,
                log_base: SAFETY_MULTIPLIER * 1181104350,
                log_byte: SAFETY_MULTIPLIER * 4399597,
                storage_write_base: SAFETY_MULTIPLIER * 21398912000,
                storage_write_key_byte: SAFETY_MULTIPLIER * 23494289,
                storage_write_value_byte: SAFETY_MULTIPLIER * 10339513,
                storage_write_evicted_byte: SAFETY_MULTIPLIER * 10705769,
                storage_read_base: SAFETY_MULTIPLIER * 18785615250,
                storage_read_key_byte: SAFETY_MULTIPLIER * 10317511,
                storage_read_value_byte: SAFETY_MULTIPLIER * 1870335,
                storage_remove_base: SAFETY_MULTIPLIER * 17824343500,
                storage_remove_key_byte: SAFETY_MULTIPLIER * 12740128,
                storage_remove_ret_value_byte: SAFETY_MULTIPLIER * 3843852,
                storage_has_key_base: SAFETY_MULTIPLIER * 18013298875,
                storage_has_key_byte: SAFETY_MULTIPLIER * 10263615,
                touching_trie_node: SAFETY_MULTIPLIER * 5367318642,
                promise_and_base: SAFETY_MULTIPLIER * 488337800,
                promise_and_per_promise: SAFETY_MULTIPLIER * 1817392,
                promise_return: SAFETY_MULTIPLIER * 186717462,
            }
        }

        fn free() -> ExtCostsConfig {
            ExtCostsConfig {
                base: 0,
                contract_compile_base: 0,
                contract_compile_bytes: 0,
                read_memory_base: 0,
                read_memory_byte: 0,
                write_memory_base: 0,
                write_memory_byte: 0,
                read_register_base: 0,
                read_register_byte: 0,
                write_register_base: 0,
                write_register_byte: 0,
                utf8_decoding_base: 0,
                utf8_decoding_byte: 0,
                utf16_decoding_base: 0,
                utf16_decoding_byte: 0,
                sha256_base: 0,
                sha256_byte: 0,
                keccak256_base: 0,
                keccak256_byte: 0,
                keccak512_base: 0,
                keccak512_byte: 0,
                ripemd160_base: 0,
                ripemd160_block: 0,
                ecrecover_base: 0,
                log_base: 0,
                log_byte: 0,
                storage_write_base: 0,
                storage_write_key_byte: 0,
                storage_write_value_byte: 0,
                storage_write_evicted_byte: 0,
                storage_read_base: 0,
                storage_read_key_byte: 0,
                storage_read_value_byte: 0,
                storage_remove_base: 0,
                storage_remove_key_byte: 0,
                storage_remove_ret_value_byte: 0,
                storage_has_key_base: 0,
                storage_has_key_byte: 0,
                touching_trie_node: 0,
                promise_and_base: 0,
                promise_and_per_promise: 0,
                promise_return: 0,
            }
        }
    }

    /// Strongly-typed representation of the fees for counting.
    #[derive(Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug, PartialOrd, Ord)]
    #[allow(non_camel_case_types)]
    pub enum ExtCosts {
        base,
        contract_compile_base,
        contract_compile_bytes,
        read_memory_base,
        read_memory_byte,
        write_memory_base,
        write_memory_byte,
        read_register_base,
        read_register_byte,
        write_register_base,
        write_register_byte,
        utf8_decoding_base,
        utf8_decoding_byte,
        utf16_decoding_base,
        utf16_decoding_byte,
        sha256_base,
        sha256_byte,
        keccak256_base,
        keccak256_byte,
        keccak512_base,
        keccak512_byte,
        ripemd160_base,
        ripemd160_block,
        ecrecover_base,
        log_base,
        log_byte,
        storage_write_base,
        storage_write_key_byte,
        storage_write_value_byte,
        storage_write_evicted_byte,
        storage_read_base,
        storage_read_key_byte,
        storage_read_value_byte,
        storage_remove_base,
        storage_remove_key_byte,
        storage_remove_ret_value_byte,
        storage_has_key_base,
        storage_has_key_byte,
        touching_trie_node,
        promise_and_base,
        promise_and_per_promise,
        promise_return,

        // NOTE: this should be the last element of the enum.
        __count,
    }

    impl ExtCosts {
        pub fn value(self, config: &ExtCostsConfig) -> Gas {
            use ExtCosts::*;
            match self {
                base => config.base,
                contract_compile_base => config.contract_compile_base,
                contract_compile_bytes => config.contract_compile_bytes,
                read_memory_base => config.read_memory_base,
                read_memory_byte => config.read_memory_byte,
                write_memory_base => config.write_memory_base,
                write_memory_byte => config.write_memory_byte,
                read_register_base => config.read_register_base,
                read_register_byte => config.read_register_byte,
                write_register_base => config.write_register_base,
                write_register_byte => config.write_register_byte,
                utf8_decoding_base => config.utf8_decoding_base,
                utf8_decoding_byte => config.utf8_decoding_byte,
                utf16_decoding_base => config.utf16_decoding_base,
                utf16_decoding_byte => config.utf16_decoding_byte,
                sha256_base => config.sha256_base,
                sha256_byte => config.sha256_byte,
                keccak256_base => config.keccak256_base,
                keccak256_byte => config.keccak256_byte,
                keccak512_base => config.keccak512_base,
                keccak512_byte => config.keccak512_byte,
                ripemd160_base => config.ripemd160_base,
                ripemd160_block => config.ripemd160_block,
                ecrecover_base => config.ecrecover_base,
                log_base => config.log_base,
                log_byte => config.log_byte,
                storage_write_base => config.storage_write_base,
                storage_write_key_byte => config.storage_write_key_byte,
                storage_write_value_byte => config.storage_write_value_byte,
                storage_write_evicted_byte => config.storage_write_evicted_byte,
                storage_read_base => config.storage_read_base,
                storage_read_key_byte => config.storage_read_key_byte,
                storage_read_value_byte => config.storage_read_value_byte,
                storage_remove_base => config.storage_remove_base,
                storage_remove_key_byte => config.storage_remove_key_byte,
                storage_remove_ret_value_byte => config.storage_remove_ret_value_byte,
                storage_has_key_base => config.storage_has_key_base,
                storage_has_key_byte => config.storage_has_key_byte,
                touching_trie_node => config.touching_trie_node,
                promise_and_base => config.promise_and_base,
                promise_and_per_promise => config.promise_and_per_promise,
                promise_return => config.promise_return,

                __count => unreachable!(),
            }
        }

        pub const fn count() -> usize {
            ExtCosts::__count as usize
        }

        pub fn name_of(index: usize) -> &'static str {
            vec![
                "base",
                "contract_compile_base",
                "contract_compile_bytes",
                "read_memory_base",
                "read_memory_byte",
                "write_memory_base",
                "write_memory_byte",
                "read_register_base",
                "read_register_byte",
                "write_register_base",
                "write_register_byte",
                "utf8_decoding_base",
                "utf8_decoding_byte",
                "utf16_decoding_base",
                "utf16_decoding_byte",
                "sha256_base",
                "sha256_byte",
                "keccak256_base",
                "keccak256_byte",
                "keccak512_base",
                "keccak512_byte",
                "ripemd160_base",
                "ripemd160_block",
                "ecrecover_base",
                "log_base",
                "log_byte",
                "storage_write_base",
                "storage_write_key_byte",
                "storage_write_value_byte",
                "storage_write_evicted_byte",
                "storage_read_base",
                "storage_read_key_byte",
                "storage_read_value_byte",
                "storage_remove_base",
                "storage_remove_key_byte",
                "storage_remove_ret_value_byte",
                "storage_has_key_base",
                "storage_has_key_byte",
                "touching_trie_node",
                "promise_and_base",
                "promise_and_per_promise",
                "promise_return",
            ][index]
        }
    }
}

pub mod profile {
    use std::fmt;
    use std::ops::{Index, IndexMut};

    use crate::config::{ActionCosts, ExtCosts};

    #[derive(Clone, PartialEq, Eq)]
    pub struct DataArray(Box<[u64; Self::LEN]>);

    impl DataArray {
        pub const LEN: usize = Cost::ALL.len();
    }

    impl Index<usize> for DataArray {
        type Output = u64;

        fn index(&self, index: usize) -> &Self::Output {
            &self.0[index]
        }
    }

    impl IndexMut<usize> for DataArray {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.0[index]
        }
    }

    /// Profile of gas consumption.
    /// When add new cost, the new cost should also be append to Cost::ALL
    #[derive(Clone, PartialEq, Eq)]
    pub struct ProfileData {
        data: DataArray,
    }

    impl Default for ProfileData {
        fn default() -> ProfileData {
            ProfileData::new()
        }
    }

    impl ProfileData {
        #[inline]
        pub fn new() -> Self {
            let costs = DataArray(Box::new([0; DataArray::LEN]));
            ProfileData { data: costs }
        }

        #[inline]
        pub fn merge(&mut self, other: &ProfileData) {
            for i in 0..DataArray::LEN {
                self.data[i] = self.data[i].saturating_add(other.data[i]);
            }
        }

        #[inline]
        pub fn add_action_cost(&mut self, action: ActionCosts, value: u64) {
            self[Cost::ActionCost { action_cost_kind: action }] =
                self[Cost::ActionCost { action_cost_kind: action }].saturating_add(value);
        }

        #[inline]
        pub fn add_ext_cost(&mut self, ext: ExtCosts, value: u64) {
            self[Cost::ExtCost { ext_cost_kind: ext }] =
                self[Cost::ExtCost { ext_cost_kind: ext }].saturating_add(value);
        }

        /// WasmInstruction is the only cost we don't explicitly account for.
        /// Instead, we compute it at the end of contract call as the difference
        /// between total gas burnt and what we've explicitly accounted for in the
        /// profile.
        ///
        /// This is because WasmInstruction is the hottest cost and is implemented
        /// with the help on the VM side, so we don't want to have profiling logic
        /// there both for simplicity and efficiency reasons.
        pub fn compute_wasm_instruction_cost(&mut self, total_gas_burnt: u64) {
            let mut value = total_gas_burnt;
            for cost in Cost::ALL {
                value = value.saturating_sub(self[*cost]);
            }
            self[Cost::WasmInstruction] = value
        }

        pub fn get_action_cost(&self, action: ActionCosts) -> u64 {
            self[Cost::ActionCost { action_cost_kind: action }]
        }

        pub fn get_ext_cost(&self, ext: ExtCosts) -> u64 {
            self[Cost::ExtCost { ext_cost_kind: ext }]
        }

        pub fn host_gas(&self) -> u64 {
            let mut host_gas = 0u64;
            for cost in Cost::ALL {
                match cost {
                    Cost::ExtCost { ext_cost_kind: e } => host_gas += self.get_ext_cost(*e),
                    _ => {}
                }
            }
            host_gas
        }

        pub fn action_gas(&self) -> u64 {
            let mut action_gas = 0u64;
            for cost in Cost::ALL {
                match cost {
                    Cost::ActionCost { action_cost_kind: a } => action_gas += self.get_action_cost(*a),
                    _ => {}
                }
            }
            action_gas
        }
    }

    impl fmt::Debug for ProfileData {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use num_rational::Ratio;
            let host_gas = self.host_gas();
            let action_gas = self.action_gas();

            writeln!(f, "------------------------------")?;
            writeln!(f, "Action gas: {}", action_gas)?;
            writeln!(f, "------ Host functions --------")?;
            for cost in Cost::ALL {
                match cost {
                    Cost::ExtCost { ext_cost_kind: e } => {
                        let d = self.get_ext_cost(*e);
                        if d != 0 {
                            writeln!(
                                f,
                                "{} -> {} [{}% host]",
                                ExtCosts::name_of(*e as usize),
                                d,
                                Ratio::new(d * 100, core::cmp::max(host_gas, 1)).to_integer(),
                            )?;
                        }
                    }
                    _ => {}
                }
            }
            writeln!(f, "------ Actions --------")?;
            for cost in Cost::ALL {
                match cost {
                    Cost::ActionCost { action_cost_kind: a } => {
                        let d = self.get_action_cost(*a);
                        if d != 0 {
                            writeln!(f, "{} -> {}", ActionCosts::name_of(*a as usize), d)?;
                        }
                    }
                    _ => {}
                }
            }
            writeln!(f, "------------------------------")?;
            Ok(())
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Cost {
        ActionCost { action_cost_kind: ActionCosts },
        ExtCost { ext_cost_kind: ExtCosts },
        WasmInstruction,
    }

    impl Cost {
        pub const ALL: &'static [Cost] = &[
            // ActionCost is unlikely to have new ones, so have it at first
            Cost::ActionCost { action_cost_kind: ActionCosts::create_account },
            Cost::ActionCost { action_cost_kind: ActionCosts::delete_account },
            Cost::ActionCost { action_cost_kind: ActionCosts::deploy_contract },
            Cost::ActionCost { action_cost_kind: ActionCosts::function_call },
            Cost::ActionCost { action_cost_kind: ActionCosts::transfer },
            Cost::ActionCost { action_cost_kind: ActionCosts::value_return },
            Cost::ActionCost { action_cost_kind: ActionCosts::new_receipt },
            Cost::ExtCost { ext_cost_kind: ExtCosts::base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_bytes },
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_block },
            Cost::ExtCost { ext_cost_kind: ExtCosts::ecrecover_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::log_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::log_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_key_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_value_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_evicted_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_key_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_value_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_key_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_ret_value_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_byte },
            Cost::ExtCost { ext_cost_kind: ExtCosts::touching_trie_node },
            Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_base },
            Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_per_promise },
            Cost::ExtCost { ext_cost_kind: ExtCosts::promise_return },
            Cost::WasmInstruction,
        ];

        pub fn index(self) -> usize {
            match self {
                Cost::ActionCost { action_cost_kind: ActionCosts::create_account } => 0,
                Cost::ActionCost { action_cost_kind: ActionCosts::delete_account } => 1,
                Cost::ActionCost { action_cost_kind: ActionCosts::deploy_contract } => 2,
                Cost::ActionCost { action_cost_kind: ActionCosts::function_call } => 3,
                Cost::ActionCost { action_cost_kind: ActionCosts::transfer } => 4,
                Cost::ActionCost { action_cost_kind: ActionCosts::value_return } => 5,
                Cost::ActionCost { action_cost_kind: ActionCosts::new_receipt } => 6,
                Cost::ActionCost { action_cost_kind: ActionCosts::__count } => unreachable!(),
                Cost::ExtCost { ext_cost_kind: ExtCosts::base } => 7,
                Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_base } => 8,
                Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_bytes } => 9,
                Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_base } => 10,
                Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_byte } => 11,
                Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_base } => 12,
                Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_byte } => 13,
                Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_base } => 14,
                Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_byte } => 15,
                Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_base } => 16,
                Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_byte } => 17,
                Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_base } => 18,
                Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_byte } => 19,
                Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_base } => 20,
                Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_byte } => 21,
                Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_base } => 22,
                Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_byte } => 23,
                Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_base } => 24,
                Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_byte } => 25,
                Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_base } => 26,
                Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_byte } => 27,
                Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_base } => 28,
                Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_block } => 29,
                Cost::ExtCost { ext_cost_kind: ExtCosts::ecrecover_base } => 30,
                Cost::ExtCost { ext_cost_kind: ExtCosts::log_base } => 31,
                Cost::ExtCost { ext_cost_kind: ExtCosts::log_byte } => 32,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_base } => 33,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_key_byte } => 34,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_value_byte } => 35,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_evicted_byte } => 36,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_base } => 37,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_key_byte } => 38,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_value_byte } => 39,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_base } => 40,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_key_byte } => 41,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_ret_value_byte } => 42,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_base } => 43,
                Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_byte } => 44,
                Cost::ExtCost { ext_cost_kind: ExtCosts::touching_trie_node } => 45,
                Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_base } => 46,
                Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_per_promise } => 47,
                Cost::ExtCost { ext_cost_kind: ExtCosts::promise_return } => 48,
                Cost::WasmInstruction => 49,
                Cost::ExtCost { ext_cost_kind: ExtCosts::__count } => unreachable!(),
            }
        }
    }

    impl Index<Cost> for ProfileData {
        type Output = u64;

        fn index(&self, index: Cost) -> &Self::Output {
            &self.data[index.index()]
        }
    }

    impl IndexMut<Cost> for ProfileData {
        fn index_mut(&mut self, index: Cost) -> &mut Self::Output {
            &mut self.data[index.index()]
        }
    }

}
