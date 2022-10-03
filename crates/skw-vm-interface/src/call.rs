use std::{
    convert::{TryInto, TryFrom},
    path::PathBuf,
    fs,
    sync::Arc,
};
use std::{
    cell::{RefCell},
    rc::Rc,
};

use crate::{
    outcome::{outcome_into_result, ExecutionResult, ViewResult},
    runtime::{init_runtime, RuntimeStandalone},
};

use skw_vm_store::Store;
use skw_vm_primitives::{
    contract_runtime::{CryptoHash, Balance, Gas},
    transaction::{Transaction, ExecutionStatus},
    account::Account,
    crypto::{KeyType, InMemorySigner},
    account_id::AccountId,
    errors::RuntimeError
};

use skw_blockchain_primitives::{
    types::{Calls, Outcome, Outcomes, StatePatch, Call},
    util::{decode_hex, unpad_size, pad_size, public_key_to_offchain_id},
    BorshDeserialize, BorshSerialize,
};

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

fn vec_to_str(buf: &Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

pub struct Caller {
    account_id: AccountId,
    runtime: Rc<RefCell<RuntimeStandalone>>,

    store: Arc<Store>,
    state_root: CryptoHash,

    wasm_files_base: String,
}

impl Caller {

    pub fn new(
        store: Arc<Store>,
        state_root: CryptoHash,
        account_id: AccountId,
        wasm_files_base: String,
    ) -> Self {
        let runtime = init_runtime(
            None,
            Some(store.clone()),
            Some(state_root),
        );

        Self { 
            account_id: account_id,
            runtime: Rc::new(RefCell::new(runtime)), 

            store: store,
            state_root: CryptoHash::default(),

            wasm_files_base: wasm_files_base.clone(),
        }
    }

     /// Look up the latest state_root
    pub fn state_root(&self) -> CryptoHash {
        (*self.runtime).borrow().state_root()
    }

    fn update_account(&mut self, signer: AccountId) {
        let runtime = init_runtime(
            None,
            Some(self.store.clone()),
            Some(self.state_root),
        );

        self.runtime = Rc::new(RefCell::new(runtime));
        self.account_id = signer.clone();
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

    // RESOLVE ACTIONS
    /// Create a new user where the signer is this user account
    pub fn create_user(&self, account_id: AccountId, amount: Balance) -> Result<ExecutionResult, RuntimeError>  {
        self.submit_transaction(
            self.transaction(account_id.clone())
                .create_account()
                .transfer(amount)
        )
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

    fn key_to_account_id(key: &[u8; 32]) -> AccountId {
        AccountId::from_bytes({
            let mut whole: [u8; 33] = [0; 33];
            let (one, two) = whole.split_at_mut(1);
            one.copy_from_slice(&[2]);
            two.copy_from_slice(&key[..]);
            whole
        }).unwrap()
    }

    pub fn write_to_file(&self, output_path: &str) {
        self.store.as_ref().save_state_to_file(output_path).unwrap();
    }

    pub fn view_account(&self, account_id: AccountId) -> Option<Account> {
        (*self.runtime)
            .borrow()
            .view_account(self.account_id.clone())
    }

    pub fn call_enclave(
        &mut self, payload: &[u8]
    ) -> Vec<u8> {    
        let mut all_outcomes: Vec<u8> = Vec::new();
        let payload_len = payload.len();
        let mut offset = 0;
    
        while offset < payload_len {
            let size = unpad_size(&payload[offset..offset + 4].try_into().unwrap());
            let call_index = unpad_size(&payload[offset + 4..offset + 8].try_into().unwrap());
    
            let params: Calls = BorshDeserialize::try_from_slice(&payload[offset + 8..offset + 4 + size]).expect("input parsing failed");
            let mut outcome_of_call = Outcomes::default();
            
            for input in params.ops.iter() {
                let origin_id = input.origin_public_key;
                let receipt_id = input.receipt_public_key;
                let origin_account_id = Caller::key_to_account_id(&origin_id);
                let receipt_account_id = Caller::key_to_account_id(&receipt_id);
    
                self.update_account(origin_account_id);
                
                let mut raw_outcome: Option<Result<ExecutionResult, RuntimeError>> = None; 
                let mut view_outcome: Option<ViewResult> = None; 
        
                match input.transaction_action {
                    
                    // "create_account"
                    0 => {
                        assert!(
                            input.amount.is_some(),
                            "amount must be provided when transaction_action is set"
                        );
        
                        raw_outcome = Some(self.create_user(
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
        
                        raw_outcome = Some(self.transfer(
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
    
                        raw_outcome = Some(self.call(
                            receipt_account_id,
                            method_str.as_str(),
                            &input.args.as_ref().unwrap()[..],
                            DEFAULT_GAS,
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
    
                        view_outcome = Some(self.view(
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
        
                        let wasm_file_name = format!("{}/{}.wasm", self.wasm_files_base.clone(), receipt_account_id.to_string());
                        let wasm_path = PathBuf::from(wasm_file_name);
                        let code = fs::read(&wasm_path).unwrap();
        
                        raw_outcome = Some(self.deploy(
                            &code,
                            receipt_account_id,
                            u128::from(input.amount.unwrap()) * 10u128.pow(24),
                        ));
                    },
                    _ => {}
                }
                
                self.state_root = self.state_root();
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
            }
            
            outcome_of_call.state_root = self.state_root;
            let mut buffer: Vec<u8> = Vec::new();
            outcome_of_call.serialize(&mut buffer).unwrap();
    
            all_outcomes.extend_from_slice(&pad_size(buffer.len() + 4)[..]);
            all_outcomes.extend_from_slice(&pad_size(call_index)[..]);
            all_outcomes.extend_from_slice(&buffer[..]);
    
            offset += 4 + size;
        }
    
        all_outcomes.clone()
    }
    
}
