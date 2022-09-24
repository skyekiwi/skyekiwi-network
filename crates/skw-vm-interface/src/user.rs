use std::cell::{RefCell};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use skw_vm_primitives::{
    crypto::{InMemorySigner, KeyType, Signer},
    account_id::AccountId,
    contract_runtime::{CryptoHash, Balance, Gas},
    transaction::Transaction,
    errors::RuntimeError,
};

use crate::{
    outcome::{outcome_into_result, ExecutionResult, ViewResult},
    runtime::{RuntimeStandalone},
};

/// A user that can sign transactions.  It includes a signer and an account id.
pub struct UserAccount {
    pub runtime: Rc<RefCell<RuntimeStandalone>>,
    pub account_id: AccountId,
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
    ) -> Self {
        let runtime = Rc::clone(runtime);
        Self { runtime, account_id }
    }

    /// Look up the latest state_root
    pub fn state_root(&self) -> CryptoHash {
        (*self.runtime).borrow().state_root()
    }

    /// Transfer yoctoNear to another account
    pub fn transfer(&self, to: AccountId, deposit: Balance) -> Result<ExecutionResult, RuntimeError>  {
        self.submit_transaction(self.transaction(to).transfer(deposit))
    }

    pub fn call(
        &self,
        receiver_id: AccountId,
        method: &str,
        args: &[u8],
        gas: Gas,
        deposit: Balance,
    ) -> Result<ExecutionResult, RuntimeError>  {
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
    ) -> Result<ExecutionResult, RuntimeError>  {
        self.submit_transaction(
            self.transaction(account_id.clone())
                .create_account()
                .transfer(deposit)
                .deploy_contract(wasm_bytes.to_vec()),
        )
    }

    fn transaction(&self, receiver_id: AccountId) -> Transaction {

        let account = (*self.runtime)
            .borrow()
            .view_account(self.account_id.clone())
            .unwrap();
 
        Transaction::new(
            self.account_id.clone(),
            receiver_id,
            account.nonce,
            CryptoHash::default(),
        )
    }

    fn submit_transaction(&self, transaction: Transaction) -> Result<ExecutionResult, RuntimeError> {
        let random_signer = InMemorySigner::from_seed(KeyType::SR25519, &[0]);
        let res = (*self.runtime).borrow_mut().resolve_tx(transaction.sign(&random_signer))?;
        (*self.runtime).borrow_mut().process_all()?;
        Ok(outcome_into_result(res.1))
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
    ) -> Result<ExecutionResult, RuntimeError>  {
        signer_user
            .submit_transaction(
                signer_user
                    .transaction(account_id.clone())
                    .create_account()
                    .transfer(amount),
            )
    }

    /// Create a new user where the signer is this user account
    pub fn create_user(&self, account_id: AccountId, amount: Balance) -> Result<ExecutionResult, RuntimeError>  {
        self.create_user_from(self, account_id, amount)
    }
}
