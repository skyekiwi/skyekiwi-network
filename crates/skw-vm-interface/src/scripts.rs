use std::{
    path::Path,
    cell::{RefCell},
    rc::Rc,
    sync::Arc,
};

use crate::{
    outcome::{ExecutionResult, ViewResult},
    runtime::init_runtime,
    user::UserAccount,
};
use skw_vm_primitives::{
    contract_runtime::{CryptoHash, AccountId, Balance}, errors::RuntimeError,
};
use skw_vm_store::{Store};

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

    pub(crate) fn init(&mut self, store: &Arc<Store>, state_root: CryptoHash, signer: AccountId) {
        let (runtime, s) = init_runtime(
            signer.clone(), 
            None,
            Some(&store),
            Some(state_root),
        );

        let account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),
            signer, 
            s
        );

        self.state_root = state_root;
        self.account = Some(account);
        self.store = Some(store.clone());
    }

    pub(crate) fn update_account(&mut self, signer: AccountId) {
        let (runtime, s) = init_runtime(
            signer.clone(),
            None,
            self.store.as_ref(),
            Some(self.state_root),
        );

        let account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),signer, s
        );

        self.account = Some(account);
    }

    pub(crate) fn create_account(&self, receiver: AccountId, deposit: Balance) -> Result<ExecutionResult, RuntimeError> {
        self.account
            .as_ref().unwrap()
            .create_user( receiver, deposit )
    }

    pub(crate) fn transfer(&self, receiver: AccountId, deposit: Balance) -> Result<ExecutionResult, RuntimeError>  {
        self.account
            .as_ref().unwrap()
            .transfer( receiver, deposit )
    }

    pub(crate) fn call(&self, receiver: AccountId, method: &str, args: &[u8], deposit: Balance) -> Result<ExecutionResult, RuntimeError>  {
        self.account
            .as_ref().unwrap()
            .call(
                receiver,
                method, 
                args,
                DEFAULT_GAS,
                deposit
            )
    }

    pub(crate) fn view_method_call(&self, receiver: AccountId, method: &str, args: &[u8]) -> ViewResult {
        self.account
            .as_ref().unwrap()
            .view( receiver, method, args )
    }

    pub(crate) fn deploy(&self, wasm_bytes: &[u8], receiver: AccountId, deposit: Balance) -> Result<ExecutionResult, RuntimeError>  {
        self.account
            .as_ref().unwrap()
            .deploy( wasm_bytes, receiver, deposit )
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
