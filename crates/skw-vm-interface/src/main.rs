use std::{
    convert::{TryInto, TryFrom},
    path::PathBuf,
    fs
};
use clap::Parser;

mod outcome;
mod runtime;
mod utils;
mod user;
mod scripts;
use scripts::Script;

use outcome::{ ExecutionResult, ViewResult};
use utils::{vec_to_str};

use skw_vm_primitives::{
    contract_runtime::CryptoHash,
    transaction::ExecutionStatus,
    account_id::AccountId, errors::RuntimeError
};

use skw_vm_store::{create_store};

use skw_blockchain_primitives::{
    types::{Calls, Outcome, Outcomes, StatePatch},
    util::{decode_hex, unpad_size, pad_size, public_key_to_offchain_id},
    BorshDeserialize, BorshSerialize,
};
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
    wasm_files_base: String,
 
    #[clap(long)]
    dump_state: bool,

    #[clap(long)]
    timings: bool,
}

fn main() {
    let cli_args = CliArgs::parse();

    if cli_args.timings {
        tracing_span_tree::span_tree().enable();
    }

    let wasm_files_base = cli_args.wasm_files_base.clone();
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
        AccountId::test(),
    );

    let decoded_call = bs58::decode(&cli_args.params.unwrap_or_default()).into_vec().unwrap();

    let mut all_outcomes: Vec<u8> = Vec::new();
    let decoded_call_len = decoded_call.len();
    let mut offset = 0;

    while offset < decoded_call_len {
        let size = unpad_size(&decoded_call[offset..offset + 4].try_into().unwrap());

        let call_index = unpad_size(&decoded_call[offset + 4..offset + 8].try_into().unwrap());
        let params: Calls = BorshDeserialize::try_from_slice(&decoded_call[offset + 8..offset + 4 + size]).expect("input parsing failed");
        let mut outcome_of_call = Outcomes::default();
        
        for input in params.ops.iter() {
            let origin_id = input.origin_public_key;
            let receipt_id = input.receipt_public_key;
            let origin_account_id = AccountId::from_bytes({
                let mut whole: [u8; 33] = [0; 33];
                let (one, two) = whole.split_at_mut(1);
                one.copy_from_slice(&[2]);
                two.copy_from_slice(&origin_id);
                whole
            }).unwrap();
            let receipt_account_id = AccountId::from_bytes({
                let mut whole: [u8; 33] = [0; 33];
                let (one, two) = whole.split_at_mut(1);
                one.copy_from_slice(&[2]);
                two.copy_from_slice(&receipt_id);
                whole
            }).unwrap();

            script.update_account(origin_account_id);
            
            let mut raw_outcome: Option<Result<ExecutionResult, RuntimeError>> = None; 
            let mut view_outcome: Option<ViewResult> = None; 
    
            match input.transaction_action {
                
                // "create_account"
                0 => {
                    assert!(
                        input.amount.is_some(),
                        "amount must be provided when transaction_action is set"
                    );
    
                    raw_outcome = Some(script.create_account(
                        receipt_account_id,
                        u128::from(input.amount.unwrap()) * 10u128.pow(24),
                    ));
                },

                // "transfer"
                1 => {
                    assert!(
                        input.amount.is_some(),
                        "amount must be provided when transaction_action is set"
                    );
    
                    raw_outcome = Some(script.transfer(
                        receipt_account_id,
                        u128::from(input.amount.unwrap()) * 10u128.pow(24),
                    ));
                },
                
                // "call"
                2 => {
                    assert!(
                        input.method.is_some(),
                        "method must be provided when transaction_action is set"
                    );
    
                    assert!(
                        input.args.is_some(),
                        "args must be provided when transaction_action is set"
                    );
    
                    let method_str = vec_to_str(&input.method.as_ref().unwrap());

                    raw_outcome = Some(script.call(
                        receipt_account_id,
                        method_str.as_str(),
                        &input.args.as_ref().unwrap()[..],
                        u128::from(input.amount.unwrap_or(0)) * 10u128.pow(24),
                    ));
                },
                // "view_method_call"
                3 => {
                    assert!(
                        input.method.is_some(),
                        "method must be provided when transaction_action is set"
                    );
    
                    assert!(
                        input.args.is_some(),
                        "args must be provided when transaction_action is set"
                    );
   
                    let method_str = vec_to_str(&input.method.as_ref().unwrap());

                    view_outcome = Some(script.view_method_call(
                        receipt_account_id,
                        method_str.as_str(),
                        &input.args.as_ref().unwrap()[..]
                    ));
                },

                // "deploy"
                4 => {
                    assert!(
                        input.amount.is_some(),
                        "amount must be provided when transaction_action is set"
                    );
    
                    let wasm_file_name = format!("{}/{}.wasm", wasm_files_base.clone(), receipt_account_id.to_string());
                    let wasm_path = PathBuf::from(wasm_file_name);
                    let code = fs::read(&wasm_path).unwrap();
    
                    raw_outcome = Some(script.deploy(
                        &code,
                        receipt_account_id,
                        u128::from(input.amount.unwrap()) * 10u128.pow(24),
                    ));
                },
                _ => {}
            }
            
            state_root = script.state_root();
            let mut execution_result: Outcome = Outcome::default();
    

            match &raw_outcome {
                Some(Ok(outcome)) => {
                    execution_result.outcome_logs = outcome.logs();
                    execution_result.outcome_receipt_ids = outcome.receipt_ids().clone();
                    execution_result.outcome_tokens_burnt = outcome.tokens_burnt();
                    execution_result.outcome_status = match outcome.status() {
                        ExecutionStatus::SuccessValue(x) => Some(x),
                        _ => None,
                    };
                },
                Some(Err(err)) => {
                    execution_result.outcome_status = Some(format!("{:?}", err).as_bytes().to_vec());
                },
                None => {} 
            }
    
            match &view_outcome {
                Some(outcome) => {
                    execution_result.view_result_log = outcome.logs();
                    let res = outcome.result();
                    execution_result.view_result = res.0;
                    execution_result.view_error = res.1;
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
    println!("{:?}", bs58::encode(all_outcomes).into_string());
}

// #[cfg(test)]
// mod test {
//     use std::rc::Rc;
//     use std::cell::RefCell;
//     use crate::{
//         user::UserAccount,
//         runtime::init_runtime,
//         utils::{str_to_account_id, to_yocto},
//     };

//     use super::*;
//     #[test]
//     fn test_dump_state_from_file() {
//         let state_root = {
//             let store = create_store();

//             let (runtime, root_signer) = init_runtime(
//                 str_to_account_id(&"modlscontrac"), 
//                 None,
//                 Some(&store),
//                 None,
//             );

//             let shared_runime = &Rc::new(RefCell::new(runtime));

//             let root_account = UserAccount::new(
//                 shared_runime,
//                 str_to_account_id(&"modlscontrac"),
//                 root_signer
//             );

//             let _ = root_account
//                 .deploy(
//                     include_bytes!("../../skw-contract-sdk/examples/status-message-collections/res/status_message_collections.wasm")
//                         .as_ref()
//                         .into(),
//                     AccountId::try_from("status".to_string()).unwrap(),
//                     to_yocto("1"),
//                 );
            
//             let _ = root_account.create_user(
//                 AccountId::try_from("alice".to_string()).unwrap(),
//                 to_yocto("100")
//             );

//             let status_account = shared_runime.borrow().view_account(str_to_account_id(&"status"));
//             let alice_account = shared_runime.borrow().view_account(str_to_account_id(&"alice"));

//             assert!(status_account.is_some());
//             assert!(alice_account.is_some());
//             store.save_state_to_file("./mock/new").unwrap();
//             root_account.state_root()
//         };

//         {

//             let store = create_store();
//             store.load_state_from_file("./mock/new").unwrap();

//             let (runtime, root_signer) = init_runtime(
//                 str_to_account_id(&"modlscontrac"), 
//                 None,
//                 Some(&store),
//                 Some(state_root),
//             );

//             let shared_runime = &Rc::new(RefCell::new(runtime));

//             let root_account = UserAccount::new(
//                 shared_runime,
//                 AccountId::try_from("modlscontrac".to_string()).unwrap(), 
//                 root_signer
//             );

//             let _ = root_account
//                 .deploy(
//                     include_bytes!("../../skw-contract-sdk/examples/status-message/res/status_message.wasm")
//                         .as_ref()
//                         .into(),
//                     AccountId::try_from("status_new".to_string()).unwrap(),
//                     to_yocto("1"),
//                 );
            
//             let _ = root_account.create_user(
//                 AccountId::try_from("alice_new".to_string()).unwrap(),
//                 to_yocto("100")
//             );

//             // existing accounts in the state store
//             let status_account =  shared_runime.borrow().view_account(str_to_account_id(&"status"));
//             let alice_account =  shared_runime.borrow().view_account(str_to_account_id(&"alice"));

//             assert!(status_account.is_some());
//             assert!(alice_account.is_some());

//             // newly created accounts in the state store
//             let status_account =  shared_runime.borrow().view_account(str_to_account_id(&"status_new"));
//             let alice_account =  shared_runime.borrow().view_account(str_to_account_id(&"alice_new"));

//             assert!(status_account.is_some());
//             assert!(alice_account.is_some());
//         };
//     }
// }