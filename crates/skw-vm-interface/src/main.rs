mod scripts;
use std::path::PathBuf;
use scripts::Script;
use std::convert::{TryInto};
use clap::Parser;
use skw_contract_sdk::{CryptoHash};
use skw_vm_interface::{ExecutionResult, ViewResult};
use skw_vm_primitives::{
    contract_runtime::{Balance, Gas},
    transaction::ExecutionStatus,
};
use borsh::{BorshSerialize, BorshDeserialize};
use std::fs;
use skw_vm_store::{create_store};

pub fn decode_hex(s: &str) -> Vec<u8> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
		.collect()
}

fn pad_size(size: usize) -> [u8; 4] {
    let mut v = [0, 0, 0, 0];
    v[3] = (size & 0xff) as u8;
    v[2] = ((size >> 8) & 0xff) as u8;
    v[1] = ((size >> 16) & 0xff) as u8;
    v[0] = ((size >> 24) & 0xff) as u8;
    v
}

fn unpad_size(size: &[u8; 4]) -> usize {
    if size.len() != 4 {
        panic!("Invalid size");
    }
    return (
        size[3] as usize | 
        ((size[2] as usize) << 8) | 
        ((size[1] as usize) << 16) | 
        ((size[0] as usize) << 24)
    ).into();
}

#[derive(clap::Parser, Debug)]
struct CliArgs {
    #[clap(long)]
    state_file: PathBuf,
    
    #[clap(long)]
    state_root: String,

    #[clap(long)]
    state_patch: Option<String>,

    #[clap(long)]
    params: Option<String>,
    
    #[clap(long)]
    dump_state: bool,

    #[clap(long)]
    timings: bool,
}

pub type StatePatch = Vec<u8>;
#[derive(Default, BorshSerialize, BorshDeserialize, Debug)]
struct InputParams {
    origin: Option<String>,
    origin_public_key: Option<[u8; 32]>,
    encrypted_egress: bool,

    transaction_action: String,
    receiver: String,
    amount: Option<u32>,
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
    state_patch: StatePatch,
}

fn main() {
    let cli_args = CliArgs::parse();

    if cli_args.timings {
        tracing_span_tree::span_tree().enable();
    }

    let state_patch: StatePatch = bs58::decode(&cli_args.state_patch.unwrap_or_default()).into_vec().unwrap();
    let mut script = Script::default();
    let mut state_root: CryptoHash = decode_hex(&cli_args.state_root.as_str())
        .try_into()
        .expect("state root invalid");


    let state_path = cli_args.state_file.to_str().expect("state path invalid");

    let store = create_store();
    match state_patch.len() {
        0 => {
            store.load_state_from_file(state_path).unwrap();
        }, 
        _ => {
            store.read_from_patch(state_path, &state_patch[..]).unwrap();
        }
    }

    script.init(
        &store,
        state_root,
        &"empty".to_string(),
    );

    let decoded_call = bs58::decode(&cli_args.params.unwrap_or_default()).into_vec().unwrap();

    let mut all_outcomes: Vec<u8> = Vec::new();
    let decoded_call_len = decoded_call.len();
    let mut offset = 0;

    while offset < decoded_call_len {
        let size = unpad_size(&decoded_call[offset..offset + 4].try_into().unwrap());

        let call_index = unpad_size(&decoded_call[offset + 4..offset + 8].try_into().unwrap());
        let params: Input = BorshDeserialize::try_from_slice(&decoded_call[offset + 8..offset + 4 + size]).expect("input parsing failed");
        let mut outcome_of_call = Outputs::default();
        
        for input in params.ops.iter() {
            script.update_account(input.origin.as_ref().unwrap_or(&"empty".to_string()));
    
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
                        u128::from(input.amount.unwrap()) * 10u128.pow(24),
                    );
                },
                "transfer" => {
                    assert!(
                        input.amount.is_some(),
                        "amount must be provided when transaction_action is set"
                    );
    
                    script.transfer(
                        &input.receiver,
                        u128::from(input.amount.unwrap()) * 10u128.pow(24),
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
                        u128::from(input.amount.unwrap_or(0)) * 10u128.pow(24),
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
                        u128::from(input.amount.unwrap()) * 10u128.pow(24),
                    );
                },
                _ => {}
            }
        
            state_root = script.state_root();
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
    
            outcome_of_call.ops.push(execution_result);
            script.sync_state_root();
        }
        
        outcome_of_call.state_root = state_root;
        let mut buffer: Vec<u8> = Vec::new();
        outcome_of_call.serialize(&mut buffer).unwrap();

        all_outcomes.extend_from_slice(&pad_size(buffer.len() + 4)[..]);
        all_outcomes.extend_from_slice(&pad_size(call_index)[..]);
        all_outcomes.extend_from_slice(&buffer[..]);

        offset += 4 + size;
    }

    if cli_args.dump_state {
        script.write_to_file(&cli_args.state_file, &mut state_root);
    }

    // println!("{:?}", all_outcomes);
    println!("{:?}", bs58::encode(all_outcomes).into_string());
}
