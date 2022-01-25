mod cache;
mod runner;
pub mod prepare;
#[cfg(test)]
mod tests;

pub use skw_vm_primitives::errors::VMError;
pub use crate::cache::create_module_instance;
pub use skw_vm_host::with_ext_cost_counter;
pub use runner::WasmiVM;
pub use runner::WasmiMemory;