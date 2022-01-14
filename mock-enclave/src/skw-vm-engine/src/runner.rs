use crate::{cache};

use skw_vm_primitives::contract_runtime::{ContractCode};
use skw_vm_primitives::fees::RuntimeFeesConfig;
use crate::externals::VMHost;

use skw_vm_primitives::errors::{
    FunctionCallError, MethodResolveError, VMError, WasmTrap,
};
use skw_vm_host::types::{PromiseResult};
use skw_vm_host::{MemoryLike, VMConfig, VMContext, VMLogic, VMOutcome, RuntimeExternal};

use wasmi::{
    MemoryInstance, MemoryRef, ModuleInstance,
    memory_units::{Pages, Bytes, size_of}
};

pub struct WasmiMemory(MemoryRef);

impl WasmiMemory {
    pub fn new(
        initial_memory_pages: u32,
        max_memory_pages: u32,
    ) -> Result<Self, VMError> {
        Ok(WasmiMemory(
            MemoryInstance::alloc(
                Pages(initial_memory_pages as usize), Some(Pages(max_memory_pages as usize))
            )
            .expect("TODO creating memory cannot fail"),
        ))
    }
}

impl MemoryLike for WasmiMemory {
    fn fits_memory(&self, offset: u64, len: u64) -> bool {
        match offset.checked_add(len) {
            None => false,
            Some(end) => size_of::<Pages>() * self.0.current_size() >= Bytes(end as usize),
        }
    }

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) {
        let offset = offset as u32;

        // TODO: handle Error::OutOfBounds
        self.0.get_into(offset, buffer).expect("Memory read error");
    }

    fn read_memory_u8(&self, offset: u64) -> u8 {

        // TODO: handle Error::OutOfBounds
        self.0.get_value(offset as u32).expect("Memory read error u8")
    }

    fn write_memory(&mut self, offset: u64, buffer: &[u8]) {

        // ref: sp-sandbox - embedded_executor
        // TODO: change type
        self.0.set(offset as u32, buffer).expect("Memory write error");
        // buffer.iter()
        //     .enumerate()
        //     .for_each(|i, v| self.0.set_value(offset + i, *v));
    }
}

fn check_method(module: &ModuleInstance, method_name: &str) -> Result<(), VMError> {
    use wasmi::{ExternVal};

    if let Some(ExternVal::Func(func)) = module.export_by_name(method_name) {
        let sig = func.signature();
        if sig.params().is_empty() && sig.return_type().is_none() {
            Ok(())
        } else {
            Err(VMError::FunctionCallError(FunctionCallError::MethodResolveError(
                MethodResolveError::MethodInvalidSignature,
            )))
        }
    } else {
        Err(VMError::FunctionCallError(FunctionCallError::MethodResolveError(
            MethodResolveError::MethodNotFound,
        )))
    }
}

pub struct WasmiVM;
impl WasmiVM {
    pub fn run(
        code: &ContractCode,
        method_name: &str,
        ext: &mut dyn RuntimeExternal,
        context: VMContext,
        wasm_config: &VMConfig,
        fees_config: &RuntimeFeesConfig,
        promise_results: &[PromiseResult],
    ) -> (Option<VMOutcome>, Option<VMError>) {
        let _span = tracing::debug_span!(
            target: "vm",
            "run_wasmi",
            "code.len" = code.code.len(),
            %method_name
        )
        .entered();

        let module = cache::create_module_instance(&code, wasm_config);

        let module = match module {
            Ok(m) => m,
            Err(e) => {
                return (None, Some(VMError::FunctionCallError(FunctionCallError::CompilationError(e))));
            }
        };

        if let Err(e) = check_method(&module, method_name) {
            return (None, Some(e));
        }


        let mut memory = WasmiMemory::new(
            wasm_config.limit_config.initial_memory_pages,
            wasm_config.limit_config.max_memory_pages,
        )
        .expect("Cannot create memory for a contract call");

        let logic = VMLogic::new(ext, context, wasm_config, fees_config, promise_results, &mut memory);
        let mut host = VMHost(logic);

        let result = module
            .invoke_export(&method_name, &[], &mut host)
            .map_err(|_| FunctionCallError::WasmTrap(WasmTrap::MemoryOutOfBounds));
        
        (Some(host.0.outcome()), match result {
            Ok(_) => None,
            Err(_) => Some(VMError::FunctionCallError(FunctionCallError::WasmTrap(WasmTrap::MemoryOutOfBounds)))
        })
    }
}
