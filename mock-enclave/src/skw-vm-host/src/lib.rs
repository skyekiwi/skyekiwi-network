mod context;
mod dependencies;
mod logic;
pub mod mocks;
pub mod serde_with;
pub mod types;
pub mod gas_counter;
pub use context::VMContext;
pub use dependencies::{RuntimeExternal, MemoryLike, ValuePtr};
pub use logic::{VMLogic, VMOutcome};

pub use skw_vm_primitives::config::*;
pub use skw_vm_primitives::profile;
pub use skw_vm_primitives::contract_runtime::ProtocolVersion;

pub use skw_vm_primitives::errors::{HostError, VMLogicError};
pub use types::ReturnData;

pub use gas_counter::with_ext_cost_counter;
