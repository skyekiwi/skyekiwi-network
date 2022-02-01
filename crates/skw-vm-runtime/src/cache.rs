use std::sync::Arc;

use skw_vm_primitives::contract_runtime::{ContractCode, CryptoHash};
use skw_vm_store::StorageError;

pub(crate) fn get_code(
    code_hash: CryptoHash,
    f: impl FnOnce() -> Result<Option<ContractCode>, StorageError>,
) -> Result<Option<Arc<ContractCode>>, StorageError> {
    let code = f()?;
    Ok(code.map(|code| {
        assert_eq!(code_hash, code.hash);
        Arc::new(code)
    }))
}
