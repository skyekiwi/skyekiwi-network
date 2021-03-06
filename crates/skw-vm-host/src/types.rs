use serde::{Deserialize, Serialize};

pub use skw_vm_primitives::contract_runtime::*;

pub type PublicKey = Vec<u8>;
pub type PromiseIndex = u64;
pub type ReceiptIndex = u64;
pub type IteratorIndex = u64;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ReturnData {
    /// Method returned some value or data.
    #[serde(with = "crate::serde_with::bytes_as_str")]
    Value(Vec<u8>),

    /// The return value of the method should be taken from the return value of another method
    /// identified through receipt index.
    ReceiptIndex(ReceiptIndex),

    /// Method hasn't returned any data or promise.
    None,
}

impl ReturnData {
    /// Function to extract value from ReturnData.
    pub fn as_value(self) -> Option<Vec<u8>> {
        match self {
            ReturnData::Value(value) => Some(value),
            _ => None,
        }
    }
}

/// When there is a callback attached to one or more contract calls the execution results of these
/// calls are available to the contract invoked through the callback.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PromiseResult {
    /// Current version of the protocol never returns `PromiseResult::NotReady`.
    NotReady,
    #[serde(with = "crate::serde_with::bytes_as_str")]
    Successful(Vec<u8>),
    Failed,
}
