use skw_vm_primitives::contract_runtime::{AccountId, Balance, BlockNumber, Gas, StorageUsage};
use skw_vm_primitives::config::ViewConfig;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
/// Context for the contract execution.
pub struct VMContext {
    /// The account id of the current contract that we are executing.
    pub current_account_id: AccountId,
    /// The account id of that signed the original transaction that led to this
    /// execution.
    pub signer_account_id: AccountId,
    /// If this execution is the result of cross-contract call or a callback then
    /// predecessor is the account that called it.
    /// If this execution is the result of direct execution of transaction then it
    /// is equal to `signer_account_id`.
    pub predecessor_account_id: AccountId,
    /// The input to the contract call.
    /// Encoded as base64 string to be able to pass input in borsh binary format.
    #[serde(with = "crate::serde_with::bytes_as_base64")]
    pub input: Vec<u8>,
    /// The current block number.
    pub block_number: BlockNumber,
    /// The current block timestamp (number of non-leap-nanoseconds since January 1, 1970 0:00:00 UTC).
    #[serde(with = "crate::serde_with::u64_dec_format")]
    pub block_timestamp: u64,

    /// The balance attached to the given account. Excludes the `attached_deposit` that was
    /// attached to the transaction.
    #[serde(with = "crate::serde_with::u128_dec_format_compatible")]
    pub account_balance: Balance,
    /// The account's storage usage before the contract execution
    pub storage_usage: StorageUsage,
    /// The balance that was attached to the call that will be immediately deposited before the
    /// contract execution starts.
    #[serde(with = "crate::serde_with::u128_dec_format_compatible")]
    pub attached_deposit: Balance,
    /// The gas attached to the call that can be used to pay for the gas fees.
    pub prepaid_gas: Gas,
    #[serde(with = "crate::serde_with::bytes_as_base58")]
    /// Initial seed for randomness
    pub random_seed: Vec<u8>,
    /// If Some, it means that execution is made in a view mode and defines its configuration.
    /// View mode means that only read-only operations are allowed.
    /// See <https://nomicon.io/Proposals/0018-view-change-method.html> for more details.
    pub view_config: Option<ViewConfig>,
    /// How many `DataReceipt`'s should receive this execution result. This should be empty if
    /// this function call is a part of a batch and it is not the last action.
    pub output_data_receivers: Vec<AccountId>,
}

impl VMContext {
    pub fn is_view(&self) -> bool {
        self.view_config.is_some()
    }
}
