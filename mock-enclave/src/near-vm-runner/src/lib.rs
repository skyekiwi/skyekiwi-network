#![doc = include_str!("../README.md")]

mod cache;
mod errors;
mod imports;
mod externals;
// mod preload;
pub mod prepare;

#[cfg(test)]
mod tests;

mod wasmi_runner;

pub use near_vm_errors::VMError;
pub use near_vm_logic::with_ext_cost_counter;

// pub use cache::get_contract_cache_key;
// pub use cache::;
// pub use preload::{ContractCallPrepareRequest, ContractCallPrepareResult, ContractCaller};
pub use wasmi_runner::WasmiVM;
