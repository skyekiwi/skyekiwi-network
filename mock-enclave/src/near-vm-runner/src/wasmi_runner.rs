use crate::cache::into_vm_result;
use crate::errors::IntoVMError;
use crate::{cache, imports};
use near_primitives::contract::ContractCode;
use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_primitives::types::CompiledContractCache;
use near_primitives::errors::RuntimeError;

use near_vm_errors::{
    CompilationError, FunctionCallError, HostError, MethodResolveError, PrepareError, VMError,
    WasmTrap,
};
use near_vm_logic::types::{PromiseResult, ProtocolVersion};
use near_vm_logic::{External, MemoryLike, VMConfig, VMContext, VMLogic, VMOutcome};
use wasmi::{
    MemoryInstance, TrapKind, ModuleInstance,
    memory_units::{Pages, Bytes}, NopExternals, ImportsBuilder
};
use parity_wasm::elements::Module;

pub struct WasmiMemory(MemoryInstance);

impl WasmiMemory {
    pub fn new(
        initial_memory_pages: usize,
        max_memory_pages: usize,
    ) -> Result<Self, VMError> {
        Ok(WasmiMemory(
            MemoryInstance::alloc(
                Pages(initial_memory_pages), Some(Pages(max_memory_pages))
            )
            .expect("TODO creating memory cannot fail"),
        ))
    }

    pub fn clone(&self) -> MemoryInstance {
        self.0.clone()
    }
}

impl MemoryLike for WasmiMemory {
    fn fits_memory(&self, offset: u64, len: u64) -> bool {
        match offset.checked_add(len) {
            None => false,
            Some(end) => self.0.current_size().byte_size() >= Bytes(end as usize),
        }
    }

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) {
        let offset = offset as u32;

        // TODO: handle Error::OutOfBounds
        self.0.get_into(offset, buffer);
    }

    fn read_memory_u8(&self, offset: u64) -> u8 {

        // TODO: handle Error::OutOfBounds
        self.0.get_value(offset as u32).expect("Memory read error u8")
    }

    fn write_memory(&mut self, offset: u64, buffer: &[u8]) {
        let offset = offset as usize;

        // ref: sp-sandbox - embedded_executor
        self.0.set(offset, buffer);
        // buffer.iter()
        //     .enumerate()
        //     .for_each(|i, v| self.0.set_value(offset + i, *v));
    }
}

impl IntoVMError for wasmi::Trap {
    fn into_vm_error(self) -> VMError {
        // These vars are not used in every cases, however, downcast below use Arc::try_unwrap
        // so we cannot clone self
        // let error_msg = self.message();
        let trap_code = self.kind();
        // if let Ok(e) = self.downcast::<VMLogicError>() {
        //     return (&e).into();
        // }

        let error = match trap_code {
            TrapKind::StackOverflow => FunctionCallError::WasmTrap(WasmTrap::StackOverflow),
            TrapKind::MemoryAccessOutOfBounds => {
                FunctionCallError::WasmTrap(WasmTrap::MemoryOutOfBounds)
            }
            TrapKind::TableAccessOutOfBounds => {
                FunctionCallError::WasmTrap(WasmTrap::MemoryOutOfBounds)
            }
            TrapKind::ElemUninitialized => {
                FunctionCallError::WasmTrap(WasmTrap::IndirectCallToNull)
            }
            TrapKind::UnexpectedSignature => {
                FunctionCallError::WasmTrap(WasmTrap::IncorrectCallIndirectSignature)
            }
            TrapKind::DivisionByZero => {
                FunctionCallError::WasmTrap(WasmTrap::IllegalArithmetic)
            }
            TrapKind::InvalidConversionToInt => {
                FunctionCallError::WasmTrap(WasmTrap::IllegalArithmetic)
            }
            TrapKind::Unreachable => FunctionCallError::WasmTrap(WasmTrap::Unreachable),
            _ => FunctionCallError::WasmTrap(WasmTrap::GenericTrap)
        };
        VMError::FunctionCallError(error)
    }
}

fn check_method(module: &ModuleInstance, method_name: &str) -> Result<(), VMError> {
    use wasmi::{ExternVal, ExternVal::Func};
    if let Some(ExternVal::Func(func)) = module.export_by_name(method_name) {
        let sig = *(func).signature();
        if sig.params().is_empty() && sig.return_type().is_empty() {
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

fn run_method(
    module: &Module,
    import: &ImportsBuilder,
    method_name: &str,
    logic: &mut VMLogic,
) -> Result<(), VMError> {
    let _span = tracing::debug_span!(target: "vm", "run_method").entered();

    let instance = {
        let _span = tracing::debug_span!(target: "vm", "run_method/instantiate").entered();
        ModuleInstance::new(
            &module,
            &import,
        )?
    };

    {
        let _span = tracing::debug_span!(target: "vm", "run_method/call").entered();

        // ref: sp-sandbox - embedded exec
        // NopExternals needs to be replaced by a GuestExternals
        // let mut externals = GuestExternals {
        //     state, defined_host_functions: & self.defined_host_functions
        // };

        instance
            .invoke_export(method_name, &[], &mut NopExternals)
            .map_err(|err| translate_runtime_error(err, logic))?
    }

    {
        let _span = tracing::debug_span!(target: "vm", "run_method/drop_instance").entered();
        drop(instance)
    }

    Ok(())
}

pub(crate) fn wasmi_vm_hash() -> u64 {
    2_619u64
}

pub(crate) fn run_wasmi_module<'a>(
    module: &Module,
    memory: &mut WasmiMemory,
    method_name: &str,
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    current_protocol_version: ProtocolVersion,
) -> (Option<VMOutcome>, Option<VMError>) {
    // Do we really need that code?
    if method_name.is_empty() {
        return (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::MethodResolveError(
                MethodResolveError::MethodEmptyName,
            ))),
        );
    }

    // Note that we don't clone the actual backing memory, just increase the RC.
    let memory_copy = memory.clone();

    let mut logic = VMLogic::new_with_protocol_version(
        ext,
        context,
        wasm_config,
        fees_config,
        promise_results,
        memory,
        current_protocol_version,
    );

    let import = imports::wasmi_import::build(memory_copy, &mut logic, current_protocol_version);

    if let Err(e) = check_method(&module, method_name) {
        return (None, Some(e));
    }

    let err = run_method(module, &import, method_name, &mut logic).err();
    (Some(logic.outcome()), err)
}

pub(crate) struct WasmiVM;

impl crate::runner::VM for WasmiVM {
    fn run(
        &self,
        code: &ContractCode,
        method_name: &str,
        ext: &mut dyn External,
        context: VMContext,
        wasm_config: &VMConfig,
        fees_config: &RuntimeFeesConfig,
        promise_results: &[PromiseResult],
        current_protocol_version: ProtocolVersion,
        cache: Option<&dyn CompiledContractCache>,
    ) -> (Option<VMOutcome>, Option<VMError>) {
        let _span = tracing::debug_span!(
            target: "vm",
            "run_wasmi",
            "code.len" = code.code().len(),
            %method_name
        )
        .entered();

        if method_name.is_empty() {
            return (
                None,
                Some(VMError::FunctionCallError(FunctionCallError::MethodResolveError(
                    MethodResolveError::MethodEmptyName,
                ))),
            );
        }

        let module =
            cache::wasmi_cache::compile_module_cached_wasmi(&code, wasm_config, cache);
        let module = match into_vm_result(module) {
            Ok(it) => it,
            Err(err) => return (None, Some(err)),
        };

        let mut memory = WasmiMemory::new(
            wasm_config.limit_config.initial_memory_pages,
            wasm_config.limit_config.max_memory_pages,
        )
        .expect("Cannot create memory for a contract call");
        // Note that we don't clone the actual backing memory, just increase the RC.
        let memory_copy = memory.clone();

        let mut logic = VMLogic::new_with_protocol_version(
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            &mut memory,
            current_protocol_version,
        );

        // TODO: remove, as those costs are incorrectly computed, and we shall account it on deployment.
        if logic.add_contract_compile_fee(code.code().len() as u64).is_err() {
            return (
                Some(logic.outcome()),
                Some(VMError::FunctionCallError(FunctionCallError::HostError(
                    near_vm_errors::HostError::GasExceeded,
                ))),
            );
        }

        let import_object =
            imports::wasmi_import::build(memory_copy, &mut logic, current_protocol_version);

        if let Err(e) = check_method(&module, method_name) {
            return (None, Some(e));
        }

        let err = run_method(&module, &import_object, method_name, &mut logic).err();
        (Some(logic.outcome()), err)
    }

    fn precompile(
        &self,
        code: &[u8],
        code_hash: &near_primitives::hash::CryptoHash,
        wasm_config: &VMConfig,
        cache: &dyn CompiledContractCache,
    ) -> Option<VMError> {
        let result = crate::cache::wasmi_cache::compile_and_serialize_wasmi(
            code,
            code_hash,
            wasm_config,
            cache,
        );
        into_vm_result(result).err()
    }

    fn check_compile(&self, code: &[u8]) -> bool {
        Module::from_buffer(code).is_ok()
    }
}
