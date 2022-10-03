use std::{
    convert::{TryInto, TryFrom},
    path::PathBuf,
    fs
};
use clap::Parser;

mod outcome;
use outcome::{ ExecutionResult, ViewResult};

use skw_vm_store::{create_store};
use skw_vm_interface::call::Caller;
use skw_vm_primitives::{
    contract_runtime::CryptoHash,
    transaction::ExecutionStatus,
    account_id::AccountId, errors::RuntimeError
};

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
    let state_root: CryptoHash = decode_hex(&cli_args.state_root.as_str())
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

    let decoded_call = bs58::decode(&cli_args.params.unwrap_or_default()).into_vec().unwrap();
    
    let mut caller = Caller::new(
        store.clone(), state_root, AccountId::test(), wasm_files_base
    );

    let all_outcomes = caller.call_enclave(&decoded_call);
    caller.write_to_file(state_path);
    println!("{:?}", bs58::encode(all_outcomes).into_string());
}

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use skw_vm_interface::{
        call::Caller,
    };
    
    fn to_yocto(value: &str) -> u128 {
        let vals: Vec<_> = value.split('.').collect();
        let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
        if vals.len() > 1 {
            let power = vals[1].len() as u32;
            let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
            part1 + part2
        } else {
            part1
        }
    }

    use super::*;
    #[test]
    fn test_dump_state_from_file() {
        let state_root = {
            let store = create_store();

            let caller = Caller::new(
                store.clone(), [0u8; 32], AccountId::system(), 
            "../../skw-contract-sdk/examples/status-message-collections/res/".to_string() );

            let _ = caller
                .deploy(
                    include_bytes!("../../skw-contract-sdk/examples/status-message-collections/res/status_message_collections.wasm")
                        .as_ref()
                        .into(),
                    AccountId::test(),
                    to_yocto("1"),
                );
            
            let _ = caller.create_user(
                AccountId::test2(),
                to_yocto("100")
            );

            let contract_account = caller.view_account(AccountId::test());
            let normal_account = caller.view_account(AccountId::test2());
            
            assert!(contract_account.is_some());
            assert!(normal_account.is_some());
            caller.write_to_file("./mock/new");
            caller.state_root()
        };

        {
            let store = create_store();
            store.load_state_from_file("./mock/new").unwrap();

            let caller = Caller::new(
                store.clone(), state_root, AccountId::system(), 
            "../../skw-contract-sdk/examples/status-message-collections/res/".to_string() );

            let contract_account = caller.view_account(AccountId::test());
            let normal_account = caller.view_account(AccountId::test2());
            
            assert!(contract_account.is_some());
            assert!(normal_account.is_some());
        };
    }
}