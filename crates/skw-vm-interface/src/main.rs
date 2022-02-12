mod scripts;
use std::path::PathBuf;
use scripts::Script;

use skw_contract_sdk::{CryptoHash};
use skw_vm_interface::{ExecutionResult, ViewResult};
use skw_vm_primitives::{
    contract_runtime::{Balance, Gas},
    transaction::ExecutionStatus,

};

use serde::{Serialize, Deserialize};
use clap::Clap;
use std::fs;

#[derive(Clap)]
struct CliArgs {
    #[clap(long)]
    state_file: Option<PathBuf>,
    #[clap(long)]
    state_root: Option<String>,

    #[clap(long)]
    signer: Option<String>,

    #[clap(long)]
    transaction_action: Option<String>,
    
    #[clap(long)]
    receiver: Option<String>,

    #[clap(long)]
    amount: Option<Balance>,
    
    #[clap(long)]
    wasm_file: Option<PathBuf>,
    
    #[clap(long)]
    method: Option<String>,

    #[clap(long)]
    args: Option<String>,

    #[clap(long)]
    to: Option<String>,
    
    #[clap(long)]
    timings: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct InterfaceOutcome {
    pub view_result_log: Vec<String>,
    pub view_result: Vec<u8>,
    pub outcome_logs: Vec<String>,
    pub outcome_receipt_ids: Vec<CryptoHash>,
    pub outcome_gas_burnt: Gas,
    pub outcome_tokens_burnt: Balance,
    pub outcome_executor_id: String,
    pub outcome_status: Option<Vec<u8>>,
    pub new_state_root: CryptoHash,
}


fn main() {
    let cli_args = CliArgs::parse();

    if cli_args.timings {
        tracing_span_tree::span_tree().enable();
    }

    let mut script = Script::default();

    if let Some(signer) = &cli_args.signer {
        script.set_signer_str(signer);
    }

    if let Some(path) = &cli_args.state_file {
        assert!(
            cli_args.state_root.is_some(),
            "state_root must be provided when state_file is set"
        );

        script.init(
            path.as_path(),
            &cli_args.state_root.unwrap(),

        );
    }

    if let Some(transaction_action) = &cli_args.transaction_action {
        assert!(
            cli_args.receiver.is_some(),
            "receiver must be provided when transaction_action is set"
        );

        let mut outcome: Option<ExecutionResult> = None; 
        let mut view_outcome: Option<ViewResult> = None; 

        let mut state_root: CryptoHash = CryptoHash::default();

        match transaction_action.as_str() {
            "create_account" => {
                assert!(
                    cli_args.amount.is_some(),
                    "amount must be provided when transaction_action is set"
                );

                script.create_account(
                    &cli_args.receiver.unwrap(),
                    cli_args.amount.unwrap(),
                );
            },
            "transfer" => {
                assert!(
                    cli_args.amount.is_some(),
                    "amount must be provided when transaction_action is set"
                );

                script.transfer(
                    &cli_args.receiver.unwrap(),
                    cli_args.amount.unwrap(),
                );
            },
            "call" => {
                assert!(
                    cli_args.method.is_some(),
                    "method must be provided when transaction_action is set"
                );

                assert!(
                    cli_args.args.is_some(),
                    "args must be provided when transaction_action is set"
                );

                outcome = Some(script.call(
                    &cli_args.receiver.unwrap(),
                    &cli_args.method.unwrap(),
                    &cli_args.args.unwrap().as_bytes(),
                    cli_args.amount.unwrap(),
                ));
            },
            "view_method_call"  => {
                assert!(
                    cli_args.method.is_some(),
                    "method must be provided when transaction_action is set"
                );

                assert!(
                    cli_args.args.is_some(),
                    "args must be provided when transaction_action is set"
                );

                view_outcome = Some(script.view_method_call(
                    &cli_args.receiver.unwrap(),
                    &cli_args.method.unwrap(),
                    &cli_args.args.unwrap().as_bytes(),
                ));
            },
            "delete_account" => {
                script.delete_account(
                    &cli_args.receiver.unwrap(),
                    &cli_args.to.unwrap(),
                );
            },
            "deploy" => {
                assert!(
                    cli_args.wasm_file.is_some(),
                    "wasm_file must be provided when transaction_action is set"
                );

                assert!(
                    cli_args.amount.is_some(),
                    "amount must be provided when transaction_action is set"
                );

                let code = fs::read(&cli_args.wasm_file.unwrap()).unwrap();

                script.deploy(
                    &code,
                    &cli_args.receiver.unwrap(),
                    cli_args.amount.unwrap(),
                );
            },
            _ => {}
        }

        // #[derive(Serialize, Deserialize, Default)]
        // struct InterfaceOutcome {
        //     pub view_result_log: Vec<String>,
        //     pub view_result: Vec<u8>,
        //     pub outcome_logs: Vec<String>,
        //     pub outcome_receipt_ids: Vec<CryptoHash>,
        //     pub outcome_gas_burnt: Gas,
        //     pub outcome_tokens_burnt: Balance,
        //     pub outcome_executor_id: String,
        //     pub outcome_status: bool,
        //     pub new_state_root: CryptoHash,
        // }

        let mut execution_result: InterfaceOutcome = InterfaceOutcome::default();

        match &outcome {
            Some(outcome) => {
                execution_result.outcome_logs = outcome.logs().clone();
                execution_result.outcome_receipt_ids = outcome.receipt_ids().clone();
                execution_result.outcome_gas_burnt = outcome.gas_burnt().0;
                execution_result.outcome_tokens_burnt = outcome.tokens_burnt();
                execution_result.outcome_executor_id = outcome.executor_id().to_string();
                execution_result.outcome_status = match outcome.status() {
                    ExecutionStatus::SuccessValue(x) => Some(x),
                    _ => None,
                };

                // println!("{:#?}", outcome);
            }
            _ => {}
        }

        match &view_outcome {
            Some(outcome) => {
                execution_result.view_result_log = outcome.logs().clone();
                execution_result.view_result = outcome.unwrap().clone();
                
                // println!("{:#?}", outcome);
            }
            _ => {}
        }

        
        if let Some(path) = &cli_args.state_file {
            script.write_to_file(&path, &mut state_root);
        }

        execution_result.new_state_root = state_root;
        // println!("{:?}", state_root);

        let output: String = serde_json::to_string(&execution_result).unwrap();
        println!("{}", output);
    }
}


// #[test]
// fn vm_script_smoke_test() {
//     use skw_vm_host::ReturnData;

//     tracing_span_tree::span_tree().enable();

//     let mut script = Script::default();
//     let contract = script.contract(near_test_contracts::rs_contract().to_vec());

//     script.step(contract, "log_something").repeat(3);
//     script.step(contract, "sum_n").input(100u64.to_le_bytes().to_vec());

//     let res = script.run();

//     assert_eq!(res.outcomes.len(), 4);

//     let logs = &res.outcomes[0].0.as_ref().unwrap().logs;
//     assert_eq!(logs, &vec!["hello".to_string()]);

//     let ret = res.outcomes.last().unwrap().0.as_ref().unwrap().return_data.clone();

//     let expected = ReturnData::Value(4950u64.to_le_bytes().to_vec());
//     assert_eq!(ret, expected);
// }

// #[test]
// fn profile_data_is_per_outcome() {
//     let mut script = Script::default();

//     let contract = script.contract(near_test_contracts::rs_contract().to_vec());

//     script.step(contract, "sum_n").input(100u64.to_le_bytes().to_vec());
//     script.step(contract, "log_something").repeat(2);
//     script.step(contract, "write_key_value");
//     let res = script.run();
//     assert_eq!(res.outcomes.len(), 4);
//     assert_eq!(
//         res.outcomes[1].0.as_ref().unwrap().profile.host_gas(),
//         res.outcomes[2].0.as_ref().unwrap().profile.host_gas()
//     );
//     assert!(
//         res.outcomes[1].0.as_ref().unwrap().profile.host_gas()
//             > res.outcomes[3].0.as_ref().unwrap().profile.host_gas()
//     );
// }

