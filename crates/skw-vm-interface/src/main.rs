mod scripts;
use std::path::PathBuf;
use scripts::Script;

use skw_contract_sdk::{CryptoHash};
use skw_vm_interface::{ExecutionResult, ViewResult};
use skw_vm_primitives::{
    contract_runtime::{Balance, Gas},
    transaction::ExecutionStatus,
};
use borsh::{BorshSerialize, BorshDeserialize};
use clap::Clap;
use std::fs;

pub fn decode_hex(s: &str) -> Vec<u8> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
		.collect()
}
#[derive(Clap)]
struct CliArgs {
    #[clap(long)]
    state_file: Option<PathBuf>,
    #[clap(long)]
    state_root: Option<String>,

    #[clap(long)]
    signer: Option<String>,

    #[clap(long)]
    params: Option<String>,
    
    #[clap(long)]
    timings: bool,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
struct InputParams {
    origin: Option<String>,
    origin_public_key: Option<[u8; 32]>,
    encrypted_egress: bool,

    transaction_action: String,
    receiver: String,
    amount: Option<Balance>,
    wasm_blob_path: Option<String>,
    method: Option<String>,
    args: Option<String>,
    to: Option<String>,
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
struct Input {
   ops: Vec<InputParams>,
}


#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
struct InterfaceOutcome {
    pub view_result_log: Vec<String>,
    pub view_result: Vec<u8>,
    pub outcome_logs: Vec<String>,
    pub outcome_receipt_ids: Vec<CryptoHash>,
    pub outcome_gas_burnt: Gas,
    pub outcome_tokens_burnt: Balance,
    pub outcome_executor_id: String,
    pub outcome_status: Option<Vec<u8>>,
}
#[derive(BorshSerialize, BorshDeserialize, Default, Debug)]
struct Outputs {
    ops: Vec<InterfaceOutcome>,
    state_root: CryptoHash,
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

    let decoded_call = bs58::decode(&cli_args.params.unwrap_or_default()).into_vec().unwrap();
    let params: Input = BorshDeserialize::try_from_slice(&decoded_call).expect("input parsing failed");

    let mut outcomes = Outputs::default();

    for input in params.ops.iter() {
        let mut outcome: Option<ExecutionResult> = None; 
        let mut view_outcome: Option<ViewResult> = None; 

        match input.transaction_action.as_str() {
            "create_account" => {
                assert!(
                    input.amount.is_some(),
                    "amount must be provided when transaction_action is set"
                );

                script.create_account(
                    &input.receiver,
                    input.amount.unwrap(),
                );
            },
            "transfer" => {
                assert!(
                    input.amount.is_some(),
                    "amount must be provided when transaction_action is set"
                );

                script.transfer(
                    &input.receiver,
                    input.amount.unwrap(),
                );
            },
            "call" => {
                assert!(
                    input.method.is_some(),
                    "method must be provided when transaction_action is set"
                );

                assert!(
                    input.args.is_some(),
                    "args must be provided when transaction_action is set"
                );

                outcome = Some(script.call(
                    &input.receiver,
                    &input.method.as_ref().unwrap(),
                    &input.args.as_ref().unwrap().as_bytes(),
                    input.amount.unwrap_or(0),
                ));
            },
            "view_method_call"  => {
                assert!(
                    input.method.is_some(),
                    "method must be provided when transaction_action is set"
                );

                assert!(
                    input.args.is_some(),
                    "args must be provided when transaction_action is set"
                );

                view_outcome = Some(script.view_method_call(
                    &input.receiver,
                    &input.method.as_ref().unwrap(),
                    &input.args.as_ref().unwrap().as_bytes(),
                ));
            },
            "delete_account" => {
                script.delete_account(
                    &input.receiver,
                    &input.to.as_ref().unwrap(),
                );
            },
            "deploy" => {
                assert!(
                    input.wasm_blob_path.is_some(),
                    "wasm_file must be provided when transaction_action is set"
                );

                assert!(
                    input.amount.is_some(),
                    "amount must be provided when transaction_action is set"
                );

                let wasm_path = PathBuf::from(input.wasm_blob_path.as_ref().unwrap());
                let code = fs::read(&wasm_path).unwrap();

                script.deploy(
                    &code,
                    &input.receiver,
                    input.amount.unwrap(),
                );
            },
            _ => {}
        }
    
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
            }
            _ => {}
        }

        match &view_outcome {
            Some(outcome) => {
                execution_result.view_result_log = outcome.logs().clone();
                execution_result.view_result = outcome.unwrap().clone();
            }
            _ => {}
        }

        outcomes.ops.push(execution_result);
    }
    
    let mut state_root = CryptoHash::default();
    if let Some(path) = &cli_args.state_file {
        script.write_to_file(&path, &mut state_root);
    }

    outcomes.state_root = state_root;

    let mut buffer: Vec<u8> = Vec::new();
    // println!("{:?}", outcomes);
    outcomes.serialize(&mut buffer).unwrap();

    println!("{:?}", state_root);
    println!("{:?}", bs58::encode(buffer).into_string());
}
