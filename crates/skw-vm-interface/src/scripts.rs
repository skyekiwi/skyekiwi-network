use std::string::String;
use std::path::{Path};
use std::num::ParseIntError;
use std::convert::{TryInto, TryFrom};
use std::cell::{RefCell};
use std::rc::Rc;
use std::sync::Arc;

use skw_vm_interface::{ExecutionResult, ViewResult};
use skw_vm_primitives::{
    contract_runtime::{CryptoHash, AccountId, Balance},
};
use skw_vm_store::{create_store, Store};
use skw_vm_interface::{
    runtime::init_runtime, UserAccount,
};

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

#[derive(Clone, Copy)]
pub struct Contract(usize);

/// Constructs a "script" to execute several contracts in a row. This is mainly
/// intended for VM benchmarking.
pub struct Script { 
    signer_str: Option<String>,
    account: Option<UserAccount>,
    store: Option<Arc<Store>>,
}

impl Default for Script {
    fn default() -> Self {
        Script { 
            signer_str: Some("root".to_string()), 
            account: None, 
            store: None, 
        }
    }
}

impl Script {

    pub(crate) fn set_signer_str(&mut self, signer_str: &String) {
        self.signer_str = Some(signer_str.clone());
    }

    pub(crate) fn init(&mut self, path: &Path, state_root: &String) {
        let state_root: CryptoHash = decode_hex(&state_root.as_str())
            .unwrap()
            .try_into()
            .expect("state root invalid");
        let state_path = path.to_str().expect("state path invalid");
        let signer_str = self.signer_str.as_ref().unwrap().as_str();

        let store = create_store();
        store.load_state_from_file(state_path).unwrap();

        let (runtime, signer) = init_runtime(
            signer_str, 
            None,
            Some(&store),
            Some(state_root),
        );

        let root_account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),
            AccountId::try_from(signer_str.to_string()).unwrap(), 
            signer
        );

        self.account = Some(root_account);
        self.store = Some(store.clone());
    }

    pub(crate) fn create_account(&self, receiver: &str, deposit: Balance) {
        self.account
            .as_ref().unwrap()
            .create_user(
                AccountId::try_from(receiver.to_string()).unwrap(),
                deposit
            );
    }

    pub(crate) fn transfer(&self, receiver: &str, deposit: Balance) {
        self.account
            .as_ref().unwrap()
            .transfer(
                AccountId::try_from(receiver.to_string()).unwrap(),
                deposit
            );
    }

    pub(crate) fn call(&self, receiver: &str, method: &str, args: &[u8], deposit: Balance) -> ExecutionResult {
        self.account
            .as_ref().unwrap()
            .call(
                AccountId::try_from(receiver.to_string()).unwrap(),
                method, 
                args,
                DEFAULT_GAS,
                deposit
            )
    }

    pub(crate) fn view_method_call(&self, receiver: &str, method: &str, args: &[u8]) -> ViewResult {
        self.account
            .as_ref().unwrap()
            .view(
                AccountId::try_from(receiver.to_string()).unwrap(),
                method, 
                args,
            )
    }

    pub(crate) fn deploy(&self, wasm_bytes: &[u8], receiver: &str, deposit: Balance) {
        self.account
            .as_ref().unwrap()
            .deploy(
                wasm_bytes,
                AccountId::try_from(receiver.to_string()).unwrap(),
                deposit,
            );
    }

    // This should not be called at all?
    pub(crate) fn delete_account(&self, receiver: &str, to: &str) {
        self.account
            .as_ref().unwrap()
            .delete_account(
                AccountId::try_from(receiver.to_string()).unwrap(),
                AccountId::try_from(to.to_string()).unwrap(),
            );
    }

    pub(crate) fn write_to_file(&self, output: &Path, state_root: &mut CryptoHash) {
        let output_path = output.to_str().expect("state path invalid");
        self.store.as_ref().unwrap().save_state_to_file(output_path).unwrap();
        * state_root = self.account.as_ref().unwrap().state_root();
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}
