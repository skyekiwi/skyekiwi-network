use crate::errors::ContractPrecompilatonResult;
use crate::prepare;
use crate::wasmi_runner::{wasmi_vm_hash};
use parity_wasm::elements::Module;
use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(not(feature = "no_cache"))]
use cached::{cached_key, SizedCache};
use near_primitives::contract::ContractCode;
use near_primitives::hash::CryptoHash;
use near_primitives::types::CompiledContractCache;
use near_vm_errors::{CacheError, CompilationError, FunctionCallError, VMError};
use near_vm_logic::{ProtocolVersion, VMConfig};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, BorshSerialize)]
enum ContractCacheKey {
    Version1 {
        code_hash: CryptoHash,
        vm_config_non_crypto_hash: u64,
        vm_hash: u64,
    },
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
enum CacheRecord {
    CompileModuleError(CompilationError),
    Code(Vec<u8>),
}

pub fn get_contract_cache_key(
    code: &ContractCode,
    config: &VMConfig,
) -> CryptoHash {
    let _span = tracing::debug_span!(target: "vm", "get_key").entered();
    let key = ContractCacheKey::Version1 {
        code_hash: *code.hash(),
        vm_config_non_crypto_hash: config.non_crypto_hash(),
        vm_hash: wasmi_vm_hash(),
    };
    near_primitives::hash::hash(&key.try_to_vec().unwrap())
}

fn cache_error(
    error: &CompilationError,
    key: &CryptoHash,
    cache: &dyn CompiledContractCache,
) -> Result<(), CacheError> {
    let record = CacheRecord::CompileModuleError(error.clone());
    let record = record.try_to_vec().unwrap();
    cache.put(&key.0, &record).map_err(|_io_err| CacheError::ReadError)?;
    Ok(())
}

pub fn into_vm_result<T>(
    res: Result<Result<T, CompilationError>, CacheError>,
) -> Result<T, VMError> {
    match res {
        Ok(Ok(it)) => Ok(it),
        Ok(Err(err)) => Err(VMError::FunctionCallError(FunctionCallError::CompilationError(err))),
        Err(cache_error) => Err(VMError::CacheError(cache_error)),
    }
}

#[derive(Default)]
pub struct MockCompiledContractCache {
    store: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MockCompiledContractCache {
    pub fn len(&self) -> usize {
        self.store.lock().unwrap().len()
    }
}

impl CompiledContractCache for MockCompiledContractCache {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
        self.store.lock().unwrap().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
        let res = self.store.lock().unwrap().get(key).cloned();
        Ok(res)
    }
}

impl fmt::Debug for MockCompiledContractCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let guard = self.store.lock().unwrap();
        let hm: &HashMap<_, _> = &*guard;
        fmt::Debug::fmt(hm, f)
    }
}

#[cfg(not(feature = "no_cache"))]
const CACHE_SIZE: usize = 128;

pub mod wasmi_cache {
    use near_primitives::contract::ContractCode;

    use super::*;

    fn compile_module_wasmi(
        code: &[u8],
        config: &VMConfig,
    ) -> Result<Module, CompilationError> {
        let _span = tracing::debug_span!(target: "vm", "compile_module_wasmi").entered();

        let prepared_code =
            prepare::prepare_contract(code, config).map_err(CompilationError::PrepareError)?;
        Module::from_bytes(prepared_code).map_err(|err| match err {
            wasmi::Error::Instantiation(_) => {
                CompilationError::WasmerCompileError { msg: err.to_string() }
            }
        })
    }

    pub(crate) fn compile_and_serialize_wasmi(
        wasm_code: &[u8],
        key: &CryptoHash,
        config: &VMConfig,
        cache: &dyn CompiledContractCache,
    ) -> Result<Result<wasmi::Module, CompilationError>, CacheError> {
        let _span = tracing::debug_span!(target: "vm", "compile_and_serialize_wasmi").entered();

        let module = match compile_module_wasmi(wasm_code, config) {
            Ok(module) => module,
            Err(err) => {
                cache_error(&err, key, cache)?;
                return Ok(Err(err));
            }
        };

        let code =
            module.to_bytes().map_err(|_e| CacheError::SerializationError { hash: key.0 })?;
        let serialized = CacheRecord::Code(code).try_to_vec().unwrap();
        cache.put(key.as_ref(), &serialized).map_err(|_io_err| CacheError::WriteError)?;
        Ok(Ok(module))
    }

    fn deserialize_wasmi(
        serialized: &[u8],
    ) -> Result<Result<Module, CompilationError>, CacheError> {
        let _span = tracing::debug_span!(target: "vm", "deserialize_wasmer2").entered();

        let record = CacheRecord::try_from_slice(serialized)
            .map_err(|_e| CacheError::DeserializationError)?;
        let serialized_module = match record {
            CacheRecord::CompileModuleError(err) => return Ok(Err(err)),
            CacheRecord::Code(code) => code,
        };
        unsafe {
            Ok(Ok(Module::fron_bytes(serialized_module.as_slice().map_err(|_e| CacheError::DeserializationError)?)))
        }
    }

    fn compile_module_cached_wasmi_impl(
        key: CryptoHash,
        wasm_code: &[u8],
        config: &VMConfig,
        cache: Option<&dyn CompiledContractCache>,
    ) -> Result<Result<Module, CompilationError>, CacheError> {
        match cache {
            None => Ok(compile_module_wasmi(wasm_code, config)),
            Some(cache) => {
                let serialized = cache.get(&key.0).map_err(|_io_err| CacheError::WriteError)?;
                match serialized {
                    Some(serialized) => deserialize_wasmi(serialized.as_slice()),
                    None => compile_and_serialize_wasmi(wasm_code, &key, config, cache),
                }
            }
        }
    }

    // #[cfg(not(feature = "no_cache"))]
    // cached_key! {
    //     MODULES: SizedCache<CryptoHash, Result<Result<Module, CompilationError>, CacheError>>
    //         = SizedCache::with_size(CACHE_SIZE);
    //     Key = {
    //         key
    //     };

    //     fn memcache_compile_module_cached_wasmi(
    //         key: CryptoHash,
    //         wasm_code: &[u8],
    //         config: &VMConfig,
    //         cache: Option<&dyn CompiledContractCache>,
    //     ) -> Result<Result<Module, CompilationError>, CacheError> = {
    //         compile_module_cached_wasmi_impl(key, wasm_code, config, cache)
    //     }
    // }

    pub(crate) fn compile_module_cached_wasmi(
        code: &ContractCode,
        config: &VMConfig,
        cache: Option<&dyn CompiledContractCache>,
    ) -> Result<Result<Module, CompilationError>, CacheError> {
        let key = get_contract_cache_key(code, config);
        // #[cfg(not(feature = "no_cache"))]
        // return memcache_compile_module_cached_wasmi(key, &code.code(), config, cache);
        #[cfg(feature = "no_cache")]
        return compile_module_cached_wasmi_impl(key, &code.code(), config, cache);
    }
}

/// Precompiles contract for the current default VM, and stores result to the cache.
/// Returns `Ok(true)` if compiled code was added to the cache, and `Ok(false)` if element
/// is already in the cache, or if cache is `None`.
pub fn precompile_contract(
    wasm_code: &ContractCode,
    config: &VMConfig,
    cache: Option<&dyn CompiledContractCache>,
) -> Result<Result<ContractPrecompilatonResult, CompilationError>, CacheError> {
    let cache = match cache {
        None => return Ok(Ok(ContractPrecompilatonResult::CacheNotAvailable)),
        Some(it) => it,
    };
    
    let key = get_contract_cache_key(wasm_code, config);
    // Check if we already cached with such a key.
    match cache.get(&key.0).map_err(|_io_error| CacheError::ReadError)? {
        // If so - do not override.
        Some(_) => return Ok(Ok(ContractPrecompilatonResult::ContractAlreadyInCache)),
        None => {}
    };
    let res = {
        wasmi_cache::compile_and_serialize_wasmi(
            wasm_code.code(),
            &key,
            config,
            cache,
        )?
        .map(|_module| ())
    };
    Ok(res.map(|()| ContractPrecompilatonResult::ContractCompiled))
}
