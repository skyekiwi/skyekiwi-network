use std::cell::{RefCell};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use skw_vm_primitives::{
    crypto::{InMemorySigner, KeyType, Signer},
    account_id::AccountId,
};

use skw_vm_primitives::{
    account::{AccessKey},
    contract_runtime::{CryptoHash, Balance, Gas},
    transaction::Transaction,
};

use crate::{
    outcome::{outcome_into_result, ExecutionResult, ViewResult},
    runtime::{RuntimeStandalone},
};

/// A user that can sign transactions.  It includes a signer and an account id.
pub struct UserAccount {
    pub runtime: Rc<RefCell<RuntimeStandalone>>,
    pub account_id: AccountId,
    pub signer: InMemorySigner,
}

impl Debug for UserAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserAccount").field("account_id", &self.account_id).finish()
    }
}

impl UserAccount {
    #[doc(hidden)]
    pub fn new(
        runtime: &Rc<RefCell<RuntimeStandalone>>,
        account_id: AccountId,
        signer: InMemorySigner,
    ) -> Self {
        let runtime = Rc::clone(runtime);
        Self { runtime, account_id, signer }
    }

    /// Look up the latest state_root
    pub fn state_root(&self) -> CryptoHash {
        (*self.runtime).borrow().state_root()
    }

    /// Transfer yoctoNear to another account
    pub fn transfer(&self, to: AccountId, deposit: Balance) -> ExecutionResult {
        self.submit_transaction(self.transaction(to).transfer(deposit))
    }

    pub fn call(
        &self,
        receiver_id: AccountId,
        method: &str,
        args: &[u8],
        gas: Gas,
        deposit: Balance,
    ) -> ExecutionResult {
        self.submit_transaction(self.transaction(receiver_id).function_call(
            method.to_string(),
            args.into(),
            gas,
            deposit,
        ))
    }

    /// Deploy a contract and create its account for `account_id`.
    pub fn deploy(
        &self,
        wasm_bytes: &[u8],
        account_id: AccountId,
        deposit: Balance,
    ) -> ExecutionResult {
        let signer =
            InMemorySigner::from_seed(account_id.clone(), KeyType::ED25519, &account_id.as_str());
        self.submit_transaction(
            self.transaction(account_id.clone())
                .create_account()
                .add_key(signer.public_key(), AccessKey::full_access())
                .transfer(deposit)
                .deploy_contract(wasm_bytes.to_vec()),
        )
    }

    fn transaction(&self, receiver_id: AccountId) -> Transaction {

        let access_key = (*self.runtime)
            .borrow()
            .view_access_key(self.account_id.clone(), &self.signer.public_key());
 
        let nonce = match access_key {
            Some(key) => {key.nonce + 1},
            None => 0
        };

        Transaction::new(
            self.account_id.clone(),
            self.signer.public_key(),
            receiver_id,
            nonce,
            CryptoHash::default(),
        )
    }

    fn submit_transaction(&self, transaction: Transaction) -> ExecutionResult {
        let res = (*self.runtime).borrow_mut().resolve_tx(transaction.sign(&self.signer)).unwrap();
        (*self.runtime).borrow_mut().process_all().unwrap();
        outcome_into_result(res.1)
    }

    pub fn view(&self, receiver_id: AccountId, method: &str, args: &[u8]) -> ViewResult {
        (*self.runtime).borrow().view_method_call(receiver_id, method, args)
    }

    /// Creates a user and is signed by the `signer_user`
    pub fn create_user_from(
        &self,
        signer_user: &UserAccount,
        account_id: AccountId,
        amount: Balance,
    ) -> ExecutionResult {
        let signer =
            InMemorySigner::from_seed(account_id.clone(), KeyType::ED25519, &account_id.as_str());
        signer_user
            .submit_transaction(
                signer_user
                    .transaction(account_id.clone())
                    .create_account()
                    .add_key(signer.public_key(), AccessKey::full_access())
                    .transfer(amount),
            )
    }

    /// Create a new user where the signer is this user account
    pub fn create_user(&self, account_id: AccountId, amount: Balance) -> ExecutionResult {
        self.create_user_from(self, account_id, amount)
    }
}
