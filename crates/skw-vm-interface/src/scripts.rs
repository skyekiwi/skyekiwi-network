use std::string::String;
use std::path::{Path};
use std::convert::{TryFrom};
use std::cell::{RefCell};
use std::rc::Rc;
use std::sync::Arc;

use skw_vm_interface::{ExecutionResult, ViewResult};
use skw_vm_primitives::{
    contract_runtime::{CryptoHash, AccountId, Balance},
};
use skw_vm_store::{Store};
use skw_vm_interface::{
    runtime::init_runtime, UserAccount,
};

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

#[derive(Clone, Copy)]
pub struct Contract(usize);

/// Constructs a "script" to execute several contracts in a row. This is mainly
/// intended for VM benchmarking.
pub struct Script { 
    account: Option<UserAccount>,
    store: Option<Arc<Store>>,
    state_root: CryptoHash,
}

impl Default for Script {
    fn default() -> Self {
        Script { 
            account: None, 
            store: None,
            state_root: CryptoHash::default(),
        }
    }
}

impl Script {

    pub(crate) fn init(&mut self, store: &Arc<Store>, state_root: CryptoHash, signer_str: &String) {
        let (runtime, signer) = init_runtime(
            signer_str, 
            None,
            Some(&store),
            Some(state_root),
        );

        let account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),
            AccountId::try_from(signer_str.to_string()).unwrap(), 
            signer
        );

        self.state_root = state_root;
        self.account = Some(account);
        self.store = Some(store.clone());
    }

    pub(crate) fn update_account(&mut self, signer_str: &String) {
        let (runtime, signer) = init_runtime(
            signer_str, 
            None,
            self.store.as_ref(),
            Some(self.state_root),
        );

        let account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),
            AccountId::try_from(signer_str.to_string()).unwrap(), 
            signer
        );

        self.account = Some(account);
    }

    pub(crate) fn create_account(&self, receiver: &str, deposit: Balance) {
        self.account
            .as_ref().unwrap()
            .create_user(
                AccountId::try_from(receiver.to_string()).unwrap(),
                deposit
            );
        // if receiver == "5gbnewrhzc2jxu7d55rbimkydk8pgk8itryftpfc8rjlkg5o" {
        //     let a = self.account.as_ref().unwrap();
        //     println!("self - {:?} {:?}", a.account_id(), a.account());
        //     println!("{:?}", a.other_account("deployer"));
        //     println!("{:?}", a.other_account("status_message_collections"));
        //     println!("{:?}", a.other_account("5gbnewrhzc2jxu7d55rbimkydk8pgk8itryftpfc8rjlkg5o"));
    
        // }
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

    pub(crate) fn state_root(&self) -> CryptoHash {
        self.account.as_ref().unwrap().state_root()
    }

    pub(crate) fn sync_state_root(&mut self) {
        self.state_root = self.state_root()
    }
}
