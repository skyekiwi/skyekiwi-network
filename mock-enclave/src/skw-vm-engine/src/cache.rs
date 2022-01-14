use crate::prepare;
use crate::imports::{WasmiImportResolver, create_builder};

use wasmi::{ModuleInstance, ModuleRef};
use skw_vm_primitives::contract_runtime::{ContractCode, CryptoHash};
use skw_vm_primitives::errors::{CacheError, CompilationError};
use skw_vm_primitives::config::{VMConfig};

use lazy_static::lazy_static;
use std::sync::{RwLock};

use lru::LruCache;

lazy_static! {
    static ref MODULE_CACHE: RwLock< LruCache<CryptoHash, wasmi::Module> > = 
        RwLock::new(LruCache::new(0));
}

pub fn create_module_instance(contract_code: &ContractCode, config: &VMConfig) -> Result<ModuleRef, CompilationError> {
    let code_hash = contract_code.hash;
    MODULE_CACHE.write().unwrap().get(&code_hash);

    // match get_module_instance(&code_hash) {
    //     Some(Ok(module_ref)) => Ok(module_ref),
    //     None => {}

    //     // we should not ever get here
    //     // toxic cache - removing
    //     Some(Err(_)) => {
    //         MODULE_CACHE.write().unwrap().pop(&code_hash);
    //     }
    // }

    let mut cache = MODULE_CACHE.write().unwrap();
    match cache.get(&code_hash).map(create_instance) {
        Some(Ok(module_ref)) => return Ok(module_ref),
        None => {}

        // we should not ever get here
        // toxic cache - removing
        Some(Err(_)) => {
            cache.pop(&code_hash);
        }
    }

    // no hit in cache - compiling
    let prepared_module = prepare::prepare_contract(&contract_code.code, config).map_err(|e| CompilationError::PrepareError(e))?;
    let module = wasmi::Module::from_buffer(prepared_module).map_err(|_| CompilationError::WasmCompileError)?;

    module.deny_floating_point()
        .map_err(|_| CompilationError::FloatingPointError)?;

    let result = create_instance(&module);
    cache.put(code_hash, module);
    
    result
}

// pub fn get_module_instance(code_hash: &CryptoHash) -> Option<Result<ModuleRef, CacheError>> {
//     MODULE_CACHE
//         .read()
//         .unwrap()
//         .peek(code_hash)
//         .map(create_instance)
// }

pub fn create_instance(module: &wasmi::Module) -> Result<ModuleRef, CompilationError> {
    let resolver = WasmiImportResolver {};
    let imports_builder = create_builder(&resolver);

    let module_instance = ModuleInstance::new(module, &imports_builder).map_err(|_| CompilationError::WasmCompileError)?;
    if module_instance.has_start() {
        return Err(CompilationError::StartFunctionError);
    }

    Ok(module_instance.not_started_instance().clone())
}

// #[cfg(not(feature = "no_cache"))]
// const CACHE_SIZE: usize = 128;

// pub mod wasmi_cache {
//     use near_primitives::contract::ContractCode;

//     use super::*;

//     fn compile_module_wasmi(
//         code: &[u8],
//         config: &VMConfig,
//     ) -> Result<Module, CompilationError> {
//         let _span = tracing::debug_span!(target: "vm", "compile_module_wasmi").entered();

//         let prepared_code =
//             prepare::prepare_contract(code, config).map_err(CompilationError::PrepareError)?;
//         Module::from_bytes(prepared_code).map_err(|err| match err {
//             wasmi::Error::Instantiation(_) => {
//                 CompilationError::WasmerCompileError { msg: err.to_string() }
//             }
//         })
//     }

//     pub(crate) fn compile_and_serialize_wasmi(
//         wasm_code: &[u8],
//         key: &CryptoHash,
//         config: &VMConfig,
//         cache: &dyn CompiledContractCache,
//     ) -> Result<Result<wasmi::Module, CompilationError>, CacheError> {
//         let _span = tracing::debug_span!(target: "vm", "compile_and_serialize_wasmi").entered();

//         let module = match compile_module_wasmi(wasm_code, config) {
//             Ok(module) => module,
//             Err(err) => {
//                 cache_error(&err, key, cache)?;
//                 return Ok(Err(err));
//             }
//         };

//         let code =
//             module.to_bytes().map_err(|_e| CacheError::SerializationError { hash: key.0 })?;
//         let serialized = CacheRecord::Code(code).try_to_vec().unwrap();
//         cache.put(key.as_ref(), &serialized).map_err(|_io_err| CacheError::WriteError)?;
//         Ok(Ok(module))
//     }

//     fn deserialize_wasmi(
//         serialized: &[u8],
//     ) -> Result<Result<Module, CompilationError>, CacheError> {
//         let _span = tracing::debug_span!(target: "vm", "deserialize_wasmer2").entered();

//         let record = CacheRecord::try_from_slice(serialized)
//             .map_err(|_e| CacheError::DeserializationError)?;
//         let serialized_module = match record {
//             CacheRecord::CompileModuleError(err) => return Ok(Err(err)),
//             CacheRecord::Code(code) => code,
//         };

//         //TODO: seems pretty safe to me :(
//         // unsafe {
//         Ok(Ok(Module::fron_bytes(serialized_module.as_slice().map_err(|_e| CacheError::DeserializationError)?)))
//         // }
//     }

//     fn compile_module_cached_wasmi_impl(
//         key: CryptoHash,
//         wasm_code: &[u8],
//         config: &VMConfig,
//         cache: Option<&dyn CompiledContractCache>,
//     ) -> Result<Result<Module, CompilationError>, CacheError> {
//         match cache {
//             None => Ok(compile_module_wasmi(wasm_code, config)),
//             Some(cache) => {
//                 let serialized = cache.get(&key.0).map_err(|_io_err| CacheError::WriteError)?;
//                 match serialized {
//                     Some(serialized) => deserialize_wasmi(serialized.as_slice()),
//                     None => compile_and_serialize_wasmi(wasm_code, &key, config, cache),
//                 }
//             }
//         }
//     }


//     // TODO: not sure what to do with this feature flag
//     #[cfg(not(feature = "no_cache"))]
//     cached_key! {
//         MODULES: SizedCache<CryptoHash, Result<Result<Module, CompilationError>, CacheError>>
//             = SizedCache::with_size(CACHE_SIZE);
//         Key = {
//             key
//         };

//         fn memcache_compile_module_cached_wasmi(
//             key: CryptoHash,
//             wasm_code: &[u8],
//             config: &VMConfig,
//             cache: Option<&dyn CompiledContractCache>,
//         ) -> Result<Result<Module, CompilationError>, CacheError> = {
//             compile_module_cached_wasmi_impl(key, wasm_code, config, cache)
//         }
//     }

//     pub(crate) fn compile_module_cached_wasmi(
//         code: &ContractCode,
//         config: &VMConfig,
//         cache: Option<&dyn CompiledContractCache>,
//     ) -> Result<Result<Module, CompilationError>, CacheError> {
//         let key = get_contract_cache_key(code, config);
//         #[cfg(not(feature = "no_cache"))]
//         return memcache_compile_module_cached_wasmi(key, &code.code(), config, cache);
//         #[cfg(feature = "no_cache")]
//         return compile_module_cached_wasmi_impl(key, &code.code(), config, cache);
//     }
// }

// /// Precompiles contract for the current default VM, and stores result to the cache.
// /// Returns `Ok(true)` if compiled code was added to the cache, and `Ok(false)` if element
// /// is already in the cache, or if cache is `None`.
// pub fn precompile_contract(
//     wasm_code: &ContractCode,
//     config: &VMConfig,
//     cache: Option<&dyn CompiledContractCache>,
// ) -> Result<Result<ContractPrecompilatonResult, CompilationError>, CacheError> {
//     let cache = match cache {
//         None => return Ok(Ok(ContractPrecompilatonResult::CacheNotAvailable)),
//         Some(it) => it,
//     };
    
//     let key = get_contract_cache_key(wasm_code, config);
//     // Check if we already cached with such a key.
//     match cache.get(&key.0).map_err(|_io_error| CacheError::ReadError)? {
//         // If so - do not override.
//         Some(_) => return Ok(Ok(ContractPrecompilatonResult::ContractAlreadyInCache)),
//         None => {}
//     };
//     let res = {
//         wasmi_cache::compile_and_serialize_wasmi(
//             wasm_code.code(),
//             &key,
//             config,
//             cache,
//         )?
//         .map(|_module| ())
//     };
//     Ok(res.map(|()| ContractPrecompilatonResult::ContractCompiled))
// }


