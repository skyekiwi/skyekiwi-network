use std::fs;
use std::path::Path;

use skw_vm_primitives::contract_runtime::ContractCode;
use skw_vm_primitives::fees::RuntimeFeesConfig;

use skw_vm_host::mocks::mock_external::MockedExternal;
use skw_vm_host::types::PromiseResult;
use skw_vm_host::{VMConfig, VMContext, VMOutcome};
use skw_vm_engine::{VMError};
use skw_vm_engine::WasmiVM;

use crate::State;

#[derive(Clone, Copy)]
pub struct Contract(usize);

/// Constructs a "script" to execute several contracts in a row. This is mainly
/// intended for VM benchmarking.
pub struct Script {
    contracts: Vec<ContractCode>,
    vm_config: VMConfig,
    initial_state: Option<State>,
    steps: Vec<Step>,
}

pub struct Step {
    contract: Contract,
    method: String,
    vm_context: VMContext,
    promise_results: Vec<PromiseResult>,
    repeat: u32,
}

pub struct ScriptResults {
    pub outcomes: Vec<(Option<VMOutcome>, Option<VMError>)>,
    pub state: MockedExternal,
}

impl Default for Script {
    fn default() -> Self {
        Script {
            contracts: Vec::new(),
            vm_config: VMConfig::test(),
            initial_state: None,
            steps: Vec::new(),
        }
    }
}

impl Script {
    pub(crate) fn contract(&mut self, code: Vec<u8>) -> Contract {
        let res = Contract(self.contracts.len());
        self.contracts.push(ContractCode::new(&code));
        res
    }

    #[allow(unused)]
    pub(crate) fn contract_from_file(&mut self, path: &Path) -> Contract {
        let data = fs::read(path).unwrap();
        self.contract(data)
    }

    pub(crate) fn vm_config(&mut self, vm_config: VMConfig) {
        self.vm_config = vm_config;
    }

    pub(crate) fn vm_config_from_file(&mut self, path: &Path) {
        let data = fs::read(path).unwrap();
        let vm_config = serde_json::from_slice(&data).unwrap();
        self.vm_config(vm_config)
    }

    pub(crate) fn initial_state(&mut self, state: State) {
        self.initial_state = Some(state);
    }

    pub(crate) fn initial_state_from_file(&mut self, path: &Path) {
        let data = fs::read(path).unwrap();
        let state = serde_json::from_slice(&data).unwrap();
        self.initial_state(state)
    }

    pub(crate) fn step(&mut self, contract: Contract, method: &str) -> &mut Step {
        self.steps.push(Step::new(contract, method.to_string()));
        self.steps.last_mut().unwrap()
    }

    pub(crate) fn run(mut self) -> ScriptResults {
        let mut external = MockedExternal::new();
        if let Some(State(trie)) = self.initial_state.take() {
            external.fake_trie = trie;
        }

        let mut outcomes = Vec::new();
        for step in &self.steps {
            for _ in 0..step.repeat {
                let res = WasmiVM::run(
                    &self.contracts[step.contract.0],
                    &step.method,
                    &mut external,
                    step.vm_context.clone(),
                    &self.vm_config,
                    &RuntimeFeesConfig::test(),
                    &step.promise_results,
                );
                outcomes.push(res);
            }
        }
        // println!("const fakeTrie = '{:?}'", external.fake_trie);
        // println!("{:?}", external.receipts);
        // println!("{:?}", external.validators);

        ScriptResults { outcomes, state: external }
    }
}

impl Step {
    fn new(contract: Contract, method: String) -> Step {
        Step {
            contract,
            method,
            vm_context: default_vm_context(),
            promise_results: Vec::new(),
            repeat: 1,
        }
    }
    pub(crate) fn context(&mut self, context: VMContext) -> &mut Step {
        self.vm_context = context;
        self
    }
    pub(crate) fn context_from_file(&mut self, path: &Path) -> &mut Step {
        let data = fs::read(path).unwrap();
        let context = serde_json::from_slice(&data).unwrap();
        self.context(context)
    }
    pub(crate) fn input(&mut self, input: Vec<u8>) -> &mut Step {
        self.vm_context.input = input;
        self
    }
    pub(crate) fn promise_results(&mut self, promise_results: Vec<PromiseResult>) -> &mut Step {
        self.promise_results = promise_results;
        self
    }
    #[allow(unused)]
    pub(crate) fn repeat(&mut self, n: u32) -> &mut Step {
        self.repeat = n;
        self
    }
}

fn default_vm_context() -> VMContext {
    VMContext {
        current_account_id: "alice".parse().unwrap(),
        signer_account_id: "bob".parse().unwrap(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "carol".parse().unwrap(),
        input: vec![],
        block_number: 1,
        block_timestamp: 1586796191203000000,
        account_balance: 10u128.pow(25),
        storage_usage: 100,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        view_config: None,
        output_data_receivers: vec![],
        epoch_height: 1,
    }
}

#[test]
fn vm_script_smoke_test() {
    use skw_vm_host::ReturnData;

    tracing_span_tree::span_tree().enable();

    let mut script = Script::default();
    let contract = script.contract(near_test_contracts::rs_contract().to_vec());

    script.step(contract, "log_something").repeat(3);
    script.step(contract, "sum_n").input(100u64.to_le_bytes().to_vec());

    let res = script.run();

    assert_eq!(res.outcomes.len(), 4);

    let logs = &res.outcomes[0].0.as_ref().unwrap().logs;
    assert_eq!(logs, &vec!["hello".to_string()]);

    let ret = res.outcomes.last().unwrap().0.as_ref().unwrap().return_data.clone();

    let expected = ReturnData::Value(4950u64.to_le_bytes().to_vec());
    assert_eq!(ret, expected);
}

#[test]
fn profile_data_is_per_outcome() {
    let mut script = Script::default();

    let contract = script.contract(near_test_contracts::rs_contract().to_vec());

    script.step(contract, "sum_n").input(100u64.to_le_bytes().to_vec());
    script.step(contract, "log_something").repeat(2);
    script.step(contract, "write_key_value");
    let res = script.run();
    assert_eq!(res.outcomes.len(), 4);
    assert_eq!(
        res.outcomes[1].0.as_ref().unwrap().profile.host_gas(),
        res.outcomes[2].0.as_ref().unwrap().profile.host_gas()
    );
    assert!(
        res.outcomes[1].0.as_ref().unwrap().profile.host_gas()
            > res.outcomes[3].0.as_ref().unwrap().profile.host_gas()
    );
}
