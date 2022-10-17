#![allow(unused_must_use)]

use core::fmt;
use std::convert::TryFrom;
use skw_vm_primitives::{
    contract_runtime::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus},
};
use skw_vm_runtime::state_viewer::errors::CallFunctionError;
use skw_blockchain_primitives::{types::Bytes, BorshDeserialize};

use std::fmt::Debug;
use std::fmt::Formatter;

/// An ExecutionResult is created by a UserAccount submitting a transaction.
/// It wraps an ExecutionOutcome which is the same object returned from an RPC call.
pub struct ExecutionResult {
    outcome: ExecutionOutcome,
}

impl Debug for ExecutionResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionResult").field("outcome", &self.outcome).finish()
    }
}

impl Default for ExecutionResult {
    fn default() -> Self {
        ExecutionResult::new(
            ExecutionOutcome::default(),
        )
    }
}

impl ExecutionResult {
    #[doc(hidden)]
    pub fn new(
        outcome: ExecutionOutcome,
    ) -> Self {
        Self { outcome }
    }

    /// Interpret the SuccessValue as a JSON value
    pub fn unwrap_json_value(&self) -> skw_contract_sdk::serde_json::Value {
        use skw_vm_primitives::transaction::ExecutionStatus::*;
        match &(self.outcome).status {
            SuccessValue(s) => skw_contract_sdk::serde_json::from_slice(s).unwrap(),
            err => panic!("Expected Success value but got: {:#?}", err),
        }
    }
    
    /// Deserialize SuccessValue from JSON
    pub fn unwrap_json<T: skw_contract_sdk::serde::de::DeserializeOwned>(&self) -> T {
        skw_contract_sdk::serde_json::from_value(self.unwrap_json_value()).unwrap()
    }
    

    /// Deserialize SuccessValue from Borsh
    pub fn unwrap_borsh<T: BorshDeserialize>(&self) -> T {
        use skw_vm_primitives::transaction::ExecutionStatus::*;
        match &(self.outcome).status {
            SuccessValue(s) => BorshDeserialize::try_from_slice(s).unwrap(),
            _ => panic!("Cannot get value of failed transaction"),
        }
    }
    

    /// Execution status. Contains the result in case of successful execution.
    /// NOTE: Should be the latest field since it contains unparsable by light client
    /// ExecutionStatus::Failure
    pub fn status(&self) -> ExecutionStatus {
        self.outcome.status.clone()
    }

    /// The amount of tokens burnt corresponding to the burnt gas amount.
    /// This value doesn't always equal to the `gas_burnt` multiplied by the gas price, because
    /// the prepaid gas price might be lower than the actual gas price and it creates a deficit.
    pub fn tokens_burnt(&self) -> u32 {
        u32::try_from(
            self.outcome.tokens_burnt / 10u128.pow(24)
        ).expect("Burnt tokens should fit in u32")
    }

    /// Logs from this transaction or receipt.
    pub fn logs(&self) -> Vec<Bytes> {
        let logs = &self.outcome.logs;
        let mut res = Vec::new();
        
        logs
            .iter()
            .map(|log| {
                res.push(log.as_bytes().to_vec());
            });
        res
    }

    /// Receipt IDs generated by this transaction or receipt.
    pub fn receipt_ids(&self) -> &Vec<CryptoHash> {
        &self.outcome.receipt_ids
    }

}

#[doc(hidden)]
pub fn outcome_into_result(
    outcome: ExecutionOutcome,
) -> ExecutionResult {
    match (outcome).status {
        ExecutionStatus::SuccessValue(_) |
        ExecutionStatus::Failure(_) => ExecutionResult::new(outcome),
        ExecutionStatus::SuccessReceiptId(_) => panic!("Unresolved ExecutionOutcome run runtime.resolve(tx) to resolve the final outcome of tx"),
        ExecutionStatus::Unknown => unreachable!()
    }
}

/// The result of a view call.  Contains the logs made during the view method call and Result value,
/// which can be unwrapped and deserialized.
#[derive(Debug)]
pub struct ViewResult {
    result: Result<Vec<u8>, CallFunctionError>,
    logs: Vec<String>,
}

impl ViewResult {
    pub fn new(result: Result<Vec<u8>, CallFunctionError>, logs: Vec<String>) -> Self {
        Self { result, logs }
    }

    /// Logs made during the view call
    pub fn logs(&self) -> Vec<Bytes> {
        let logs = &self.logs;
        let mut res = Vec::new();
        
        logs
            .iter()
            .map(|log| {
                res.push(log.as_bytes().to_vec());
            });
        res
    }

    /// Attempt unwrap the value returned by the view call and panic if it is an error
    pub fn result(&self) -> (Option<Bytes>, Option<Bytes>) {
        let mut res = (None, None);
        match &self.result {
            Ok(x) => {
                res.0 = Some(x.clone());
                res.1 = None;
            },
            Err(e) => {
                res.0 = None;
                res.1 = Some(e.to_string().as_bytes().to_vec());
            }
        }

        res
    }
}
