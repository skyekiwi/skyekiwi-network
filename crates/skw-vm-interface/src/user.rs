use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use skw_vm_primitives::{
    crypto::{InMemorySigner, KeyType, Signer},
    account_id::AccountId,
};
use skw_vm_store::Store;

use crate::runtime::init_runtime;
pub use crate::to_yocto;
use crate::new_p_account;

use skw_vm_primitives::{
    account::{Account, AccessKey},
    contract_runtime::{CryptoHash, Balance, Gas},
    transaction::Transaction,
};

use crate::{
    outcome_into_result,
    runtime::{GenesisConfig, RuntimeStandalone},
    ExecutionResult, ViewResult,
};

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_AMOUNT: u128 = 50_000_000_000_000_000_000_000_000;

type Runtime = Rc<RefCell<RuntimeStandalone>>;

pub struct UserTransaction {
    transaction: Transaction,
    signer: InMemorySigner,
    runtime: Runtime,
}

impl UserTransaction {
    /// Sign and execute the transaction
    pub fn submit(self) -> ExecutionResult {
        let res =
            (*self.runtime).borrow_mut().resolve_tx(self.transaction.sign(&self.signer)).unwrap();
        (*self.runtime).borrow_mut().process_all().unwrap();
        outcome_into_result(res, &self.runtime)
    }

    /// Create account for the receiver of the transaction.
    pub fn create_account(mut self) -> Self {
        self.transaction = self.transaction.create_account();
        self
    }

    /// Deploy Wasm binary
    pub fn deploy_contract(mut self, code: Vec<u8>) -> Self {
        self.transaction = self.transaction.deploy_contract(code);
        self
    }

    /// Execute contract call to receiver
    pub fn function_call(
        mut self,
        function_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    ) -> Self {
        self.transaction = self.transaction.function_call(function_name, args, gas, deposit);
        self
    }

    /// Transfer deposit to receiver
    pub fn transfer(mut self, deposit: Balance) -> Self {
        self.transaction = self.transaction.transfer(deposit);
        self
    }

    /// Delete an account and send remaining balance to `beneficiary_id`
    pub fn delete_account(mut self, beneficiary_id: &str) -> Self {
        let beneficiary_id = new_p_account(beneficiary_id);
        self.transaction = self.transaction.delete_account(beneficiary_id);
        self
    }
}

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

    /// Returns a copy of the `account_id`
    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }
    /// Look up the account information on chain.
    pub fn account(&self) -> Option<Account> {
        (*self.runtime).borrow().view_account(&self.account_id.as_str())
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
    /// Note: You will most likely not be using this method directly but rather the [`deploy!`](./macro.deploy.html) macro.
    pub fn deploy(
        &self,
        wasm_bytes: &[u8],
        account_id: AccountId,
        deposit: Balance,
    ) -> UserAccount {
        let signer =
            InMemorySigner::from_seed(new_p_account(&account_id.as_str()), KeyType::ED25519, &account_id.as_str());
        self.submit_transaction(
            self.transaction(account_id.clone())
                .create_account()
                .add_key(signer.public_key(), AccessKey::full_access())
                .transfer(deposit)
                .deploy_contract(wasm_bytes.to_vec()),
        )
        .assert_success();
        UserAccount::new(&self.runtime, account_id, signer)
    }

    pub fn deploy_and_init(
        &self,
        wasm_bytes: &[u8],
        account_id: AccountId,
        method: &str,
        args: &[u8],
        deposit: Balance,
        gas: Gas,
    ) -> UserAccount {
        let signer =
            InMemorySigner::from_seed(new_p_account(&account_id.as_str()), KeyType::ED25519, &account_id.as_str());
        self.submit_transaction(
            self.transaction(account_id.clone())
                .create_account()
                .add_key(signer.public_key(), AccessKey::full_access())
                .transfer(deposit)
                .deploy_contract(wasm_bytes.to_vec())
                .function_call(method.to_string(), args.to_vec(), gas, 0),
        )
        .assert_success();
        UserAccount::new(&self.runtime, account_id, signer)
    }

    fn transaction(&self, receiver_id: AccountId) -> Transaction {
        let nonce = (*self.runtime)
            .borrow()
            .view_access_key(&self.account_id.as_str(), &self.signer.public_key())
            .unwrap()
            .nonce + 1;
        Transaction::new(
            new_p_account(&self.account_id().as_str()),
            self.signer.public_key(),
            new_p_account(&receiver_id.as_str()),
            nonce,
            CryptoHash::default(),
        )
    }

    /// Create a user transaction to `receiver_id` to be signed the current user
    pub fn create_transaction(&self, receiver_id: AccountId) -> UserTransaction {
        let transaction = self.transaction(receiver_id);
        let runtime = Rc::clone(&self.runtime);
        UserTransaction { transaction, signer: self.signer.clone(), runtime }
    }

    fn submit_transaction(&self, transaction: Transaction) -> ExecutionResult {
        let res = (*self.runtime).borrow_mut().resolve_tx(transaction.sign(&self.signer)).unwrap();
        (*self.runtime).borrow_mut().process_all().unwrap();
        outcome_into_result(res, &self.runtime)
    }

    pub fn view(&self, receiver_id: AccountId, method: &str, args: &[u8]) -> ViewResult {
        (*self.runtime).borrow().view_method_call(&receiver_id.as_str(), method, args)
    }

    /// Creates a user and is signed by the `signer_user`
    pub fn create_user_from(
        &self,
        signer_user: &UserAccount,
        account_id: AccountId,
        amount: Balance,
    ) -> UserAccount {
        let signer =
            InMemorySigner::from_seed(new_p_account(&account_id.as_str()), KeyType::ED25519, &account_id.as_str());
        signer_user
            .submit_transaction(
                signer_user
                    .transaction(account_id.clone())
                    .create_account()
                    .add_key(signer.public_key(), AccessKey::full_access())
                    .transfer(amount),
            )
            .assert_success();
        UserAccount::new(&self.runtime, account_id, signer)
    }

    pub fn delete_account(
        &self,
        account_id: AccountId,
        beneficiary_id: AccountId,
    ) -> UserAccount {
        let signer =
            InMemorySigner::from_seed(new_p_account(&account_id.as_str()), KeyType::ED25519, &account_id.as_str());
        self.submit_transaction(
                self.transaction(account_id.clone())
                    .delete_account(beneficiary_id),
            ).assert_success();
        UserAccount::new(&self.runtime, account_id, signer)
    }


    /// Create a new user where the signer is this user account
    pub fn create_user(&self, account_id: AccountId, amount: Balance) -> UserAccount {
        self.create_user_from(self, account_id, amount)
    }

    /// Returns a reference to a memory location of the standalone runtime.
    ///
    /// # Examples
    /// ```
    /// let master_account = skw_vm_interface::init_node(None);
    /// let runtime = master_account.borrow_runtime();
    ///
    /// // with use
    /// let _block = runtime.current_block();
    /// ```
    pub fn borrow_runtime(&self) -> Ref<RuntimeStandalone> {
        (*self.runtime).borrow()
    }

    /// Returns a mutable memory location to the standalone runtime.
    ///
    /// # Examples
    /// ```
    /// let master_account = skw_vm_interface::init_node(None);
    /// let mut runtime = master_account.borrow_runtime_mut();
    ///
    /// // with use
    /// runtime.produce_block().unwrap();
    /// ```
    pub fn borrow_runtime_mut(&self) -> RefMut<RuntimeStandalone> {
        (*self.runtime).borrow_mut()
    }
}

/// The simulator takes an optional GenesisConfig, which sets up the fees and other settings.
/// It returns the `master_account` which can then create accounts and deploy contracts.
pub fn init_node(genesis_config: Option<GenesisConfig>) -> UserAccount {
    let (runtime, signer) = init_runtime(&"root", genesis_config, None, None);
    UserAccount::new(&Rc::new(RefCell::new(runtime)), AccountId::from_str("root").unwrap(), signer)
}

pub fn recover_node(genesis_config: Option<GenesisConfig>, store: Option<&Arc<Store>>, state_root: Option<CryptoHash> ) -> UserAccount {
    let (runtime, signer) = init_runtime(&"root", genesis_config, store, state_root);
    UserAccount::new(&Rc::new(RefCell::new(runtime)), AccountId::from_str("root").unwrap(), signer)
}

pub fn init_node_with_store(genesis_config: Option<GenesisConfig>, store: Option<&Arc<Store>>, state_root: Option<CryptoHash> ) -> UserAccount {
    let (runtime, signer) = init_runtime(&"root", genesis_config, store, state_root);
    UserAccount::new(&Rc::new(RefCell::new(runtime)), AccountId::from_str("root").unwrap(), signer)
}
