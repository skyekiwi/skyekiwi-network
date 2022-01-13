use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use near_primitives::contract::ContractCode;
use threadpool::ThreadPool;

use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_primitives::types::CompiledContractCache;
use near_vm_errors::VMError;
use near_vm_logic::types::PromiseResult;
use near_vm_logic::{External, ProtocolVersion, VMConfig, VMContext, VMOutcome};

use crate::cache::{self, into_vm_result};
use crate::wasmi_runner::{run_wasmi_module, WasmiMemory};

struct VMCallData {
    result: Result<wasmi::Module, VMError>,
}

struct CallInner {
    rx: Receiver<VMCallData>,
}
pub struct ContractCallPrepareRequest {
    pub code: Arc<ContractCode>,
    pub cache: Option<Arc<dyn CompiledContractCache>>,
}

#[derive(Clone)]
pub struct ContractCallPrepareResult {
    handle: usize,
}

pub struct ContractCaller {
    pool: ThreadPool,
    vm_config: VMConfig,
    vm_data: wasmi::MemoryInstance,
    preloaded: Vec<CallInner>,
}

impl ContractCaller {
    pub fn new(num_threads: usize, vm_config: VMConfig) -> ContractCaller {

        let data = WasmiMemory::new(
            vm_config.limit_config.initial_memory_pages,
            vm_config.limit_config.max_memory_pages,
        );

        ContractCaller {
            pool: ThreadPool::new(num_threads),
            vm_config,
            vm_data: data,
            preloaded: Vec::new(),
        }
    }

    pub fn preload(
        &mut self,
        requests: Vec<ContractCallPrepareRequest>,
    ) -> Vec<ContractCallPrepareResult> {
        let mut result: Vec<ContractCallPrepareResult> = Vec::new();
        for request in requests {
            let index = self.preloaded.len();
            let (tx, rx) = channel();
            self.preloaded.push(CallInner { rx });
            self.pool.execute({
                let tx = tx.clone();
                let vm_config = self.vm_config.clone();
                move || preload_in_thread(request, vm_config, tx)
            });
            result.push(ContractCallPrepareResult { handle: index });
        }
        result
    }

    pub fn run_preloaded<'a>(
        self: &mut ContractCaller,
        preloaded: &ContractCallPrepareResult,
        method_name: &str,
        ext: &mut dyn External,
        context: VMContext,
        fees_config: &'a RuntimeFeesConfig,
        promise_results: &'a [PromiseResult],
        current_protocol_version: ProtocolVersion,
    ) -> (Option<VMOutcome>, Option<VMError>) {
        match self.preloaded.get(preloaded.handle) {
            Some(call) => {
                let call_data = call.rx.recv().unwrap();
                match call_data.result {
                    Err(err) => (None, Some(err)),
                    Ok(module) => {
                        let mut new_memory;
                        let &mut memory = &mut self.vm_data;
                        run_wasmi_module(
                            &module,
                            if memory.is_some() {
                                memory.as_mut().unwrap()
                            } else {
                                new_memory = WasmiMemory::new(
                                    self.vm_config.limit_config.initial_memory_pages,
                                    self.vm_config.limit_config.max_memory_pages,
                                )
                                .unwrap();
                                &mut new_memory
                            },
                            method_name,
                            ext,
                            context,
                            &self.vm_config,
                            fees_config,
                            promise_results,
                            current_protocol_version,
                        )
                    }
                }
            }
            None => panic!("Must be valid"),
        }
    }
}

impl Drop for ContractCaller {
    fn drop(&mut self) {
        self.pool.join();
    }
}

fn preload_in_thread(
    request: ContractCallPrepareRequest,
    vm_config: VMConfig,
    tx: Sender<VMCallData>,
) {
    let cache = request.cache.as_deref();
    let module = cache::wasmi_cache::compile_module_cached_wasmi(
        &request.code,
        &vm_config,
        cache
    );
    let result = into_vm_result(module);
    tx.send(VMCallData { result }).unwrap();
}
