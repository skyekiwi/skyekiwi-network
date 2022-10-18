use crate::prepare;

use skw_vm_host::{WasmiImportResolver, create_builder};
use wasmi::{ModuleInstance, ModuleRef, MemoryRef};
use skw_vm_primitives::contract_runtime::{ContractCode, CryptoHash};
use skw_vm_primitives::errors::{CompilationError};
use skw_vm_primitives::config::{VMConfig};

use lazy_static::lazy_static;
use std::sync::{RwLock};

use lru::LruCache;

lazy_static! {
    static ref MODULE_CACHE: RwLock< LruCache<CryptoHash, wasmi::Module> > = 
        RwLock::new(LruCache::new(0));
}

pub fn create_module_instance(contract_code: &ContractCode, config: &VMConfig, memory: MemoryRef) -> Result<ModuleRef, CompilationError> {
    let code_hash = contract_code.hash;
    MODULE_CACHE.write().unwrap().get(&code_hash);

    let mut cache = MODULE_CACHE.write().unwrap();
    match cache.get(&code_hash) {
        Some(m) => {
            create_instance(&m, memory)
                .map_err(|e| {
                    cache.pop(&code_hash);
                    e
                })
        },
        None => {
            let prepared_module = prepare::prepare_contract(&contract_code.code, config).map_err(|e| CompilationError::PrepareError(e))?;
            let module = wasmi::Module::from_parity_wasm_module(prepared_module).map_err(|_| {
                CompilationError::WasmCompileError
            })?;

            // module.deny_floating_point()
            //     .map_err(|_| CompilationError::FloatingPointError)?;

            let result = create_instance(&module, memory);
            cache.put(code_hash, module);

            result
        }
    }
}

pub fn create_instance(module: &wasmi::Module, memory: MemoryRef) -> Result<ModuleRef, CompilationError> {
    let resolver = WasmiImportResolver::new(memory);
    let imports_builder = create_builder(&resolver);

    let module_instance = ModuleInstance::new(module, &imports_builder).map_err(|_| {
        CompilationError::WasmCompileError
    })?;
    if module_instance.has_start() {
        return Err(CompilationError::StartFunctionError);
    }

    Ok(module_instance.not_started_instance().clone())
}
