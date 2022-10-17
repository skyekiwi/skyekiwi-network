use std::{
    convert::TryInto,
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

use skw_vm_store::{Store, create_store};
use skw_vm_primitives::{
    contract_runtime::{CryptoHash, Balance, Gas},
    transaction::{Transaction, ExecutionStatus},
    account::Account,
    crypto::{KeyType, InMemorySigner},
    account_id::AccountId,
    errors::RuntimeError
};

use skw_blockchain_primitives::{
    types::{Calls, Outcome, Outcomes},
    util::{unpad_size, pad_size},
    BorshDeserialize, BorshSerialize,
};

use skw_contract_sdk::PendingContractTx;
use skw_contract_sdk::types::AccountId as SmallAccountId;

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_AMOUNT: u128 = 50_000_000_000_000_000_000_000_000;

fn vec_to_str(buf: &Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn small_account_id_to_account_id(account_id: SmallAccountId) -> AccountId {
    AccountId::from_bytes(
        account_id.as_bytes().try_into().unwrap()
    ).unwrap()
}

pub struct Caller {
    account_id: AccountId,
    runtime: Rc<RefCell<RuntimeStandalone>>,

    store: Arc<Store>,
    state_root: CryptoHash,

    wasm_files_base: String,
}

impl Caller {

    pub fn new_test_env(
        account_id: AccountId,
        wasm_files_base: String
    ) -> Self {
        let c = Self::new(
            create_store(),
            CryptoHash::default(),
            AccountId::root(),
            wasm_files_base
        );

        c.create_user(account_id, 1_000_000_000);

        c
    }

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

    pub fn clone_and_set(&self, signer: AccountId) -> Self {
        let runtime = init_runtime(
            None,
            Some(self.store.clone()),
            Some(self.state_root),
        );

        Self {
            account_id: signer.clone(),
            runtime: Rc::new(RefCell::new(runtime)),

            store: self.store.clone(),
            state_root: self.state_root.clone(),

            wasm_files_base: self.wasm_files_base.clone()
        }
    }

    pub fn set_account(&mut self, signer: AccountId) {
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
            account.nonce + 1,
            CryptoHash::default(),
        )
    }

    fn submit_transaction(&self, transaction: Transaction) -> Result<ExecutionResult, RuntimeError> {
        let random_signer = InMemorySigner::from_seed(KeyType::SR25519, &[0]);
        let res = (*self.runtime).borrow_mut().resolve_tx(transaction.sign(&random_signer))?;
        (*self.runtime).borrow_mut().process_all()?;
        Ok(outcome_into_result(res.1))
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }

    pub fn account(&self) -> Option<Account> {
        self.view_account(self.account_id.clone())
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

     /// Make a contract call.  `pending_tx` includes the receiver, the method to call as well as its arguments.
    /// Note: You will most likely not be using this method directly but rather the [`call!`](./macro.call.html) macro.
    pub fn function_call(
        &self,
        pending_tx: PendingContractTx,
        deposit: Balance,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.call(
            small_account_id_to_account_id(pending_tx.receiver_id.clone()), 
            &pending_tx.method, &pending_tx.args, 300000000000000, deposit
        )
    }

    /// Call a view method on a contract.
    /// Note: You will most likely not be using this method directly but rather the [`view!`](./macros.view.html) macro.
    pub fn view_method_call(&self, pending_tx: PendingContractTx) -> ViewResult {
        self.view(
            small_account_id_to_account_id(pending_tx.receiver_id.clone()), 
            &pending_tx.method, &pending_tx.args
        )
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

    // Misc Functions
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
            .view_account(account_id.clone())
    }

    // High level call wrapper
    pub fn call_payload(
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
    
                self.set_account(origin_account_id);
                
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

// pub struct ContractCaller<T> {
//     pub base_caller: Caller,
//     pub contract: T
// }

// impl<T> ContractCaller<T> {
//     pub fn new(
//         contract: T,
//         contract_id: AccountId,
//         wasm_bytes: &[u8],
//         caller: Caller,
//     ) -> Self {
//         caller.deploy(wasm_bytes, account_id, super::STORAGE_AMOUNT);
//         let contract_caller = caller.clone_and_set(contract_id);
//         Self {
//             base_caller: contract_caller,
//             contract: contract {
//                 account_id: skw_contract_sdk::types::AccountId::new(contract_id.as_bytes()[..].to_vec().try_into().unwrap())
//             }
//         }
//     }
// }

// #[macro_export]
// macro_rules! deploy {
//     ($contract: ident, $account_id:expr, $wasm_bytes: expr, $user:expr $(,)?) => {
//         deploy!($contract, $account_id, $wasm_bytes, $user, skw_vm_interface::STORAGE_AMOUNT)
//     };
//     ($contract: ident, $account_id:expr, $wasm_bytes: expr, $user:expr, $deposit: expr $(,)?) => {
//         skw_vm_interface::ContractCaller::new(
//             $contract, $account_id, $wasm_bytes, $user
//         )
//     };
//     (contract: $contract: ident, contract_id: $account_id:expr, bytes: $wasm_bytes: expr, signer_account: $user:expr $(,)?) => {
//       deploy!($contract, $account_id, $wasm_bytes, $user)
//     };
//     (contract: $contract: ident, contract_id: $account_id:expr, bytes: $wasm_bytes: expr, signer_account: $user:expr, deposit: $deposit: expr $(,)?) => {
//         deploy!($contract, $account_id, $wasm_bytes, $user, $deposit)
//     };

//     (contract: $contract: ident, contract_id: $account_id:expr, bytes: $wasm_bytes: expr, signer_account: $user:expr, gas: $gas:expr, init_method: $method: ident($($arg:expr),*) $(,)?) => {
//        deploy!($contract, $account_id, $wasm_bytes, $user, skw_vm_interface::STORAGE_AMOUNT, $gas, $method, $($arg),*)
//     };
//     (contract: $contract: ident, contract_id: $account_id:expr, bytes: $wasm_bytes: expr, signer_account: $user:expr, deposit: $deposit: expr, init_method: $method: ident($($arg:expr),*) $(,)?) => {
//        deploy!($contract, $account_id, $wasm_bytes, $user, $deposit, skw_vm_interface::DEFAULT_GAS, $method, $($arg),*)
//     };
//     (contract: $contract: ident, contract_id: $account_id:expr, bytes: $wasm_bytes: expr, signer_account: $user:expr, init_method: $method: ident($($arg:expr),*) $(,)?) => {
//        deploy!($contract, $account_id, $wasm_bytes, $user, skw_vm_interface::STORAGE_AMOUNT, skw_vm_interface::DEFAULT_GAS, $method, $($arg),*)
//     };
// }

// #[macro_export]
// macro_rules! call {
//     ($signer:expr, $deposit: expr, $gas: expr, $contract: ident, $method:ident, $($arg:expr),*) => {
//         $signer.function_call((&$contract).contract.$method($($arg),*), $gas, $deposit)
//     };
//     ($signer:expr, $contract: ident.$method:ident($($arg:expr),*), $deposit: expr, $gas: expr) => {
//         call!($signer, $deposit, $gas, $contract, $method, $($arg),*)
//     };
//     ($signer:expr, $contract: ident.$method:ident($($arg:expr),*)) => {
//         call!($signer, 0, skw_vm_interface::DEFAULT_GAS,  $contract, $method, $($arg),*)
//     };
//     ($signer:expr, $contract: ident.$method:ident($($arg:expr),*), gas=$gas_or_deposit: expr) => {
//         call!($signer, 0, $gas_or_deposit, $contract, $method, $($arg),*)
//     };
//     ($signer:expr, $contract: ident.$method:ident($($arg:expr),*), deposit=$gas_or_deposit: expr) => {
//         call!($signer, $gas_or_deposit, skw_vm_interface::DEFAULT_GAS, $contract, $method, $($arg),*)
//     };
// }

// #[macro_export]
// macro_rules! view {
//     ($contract: ident.$method:ident($($arg:expr),*)) => {
//         (&$contract).base_caller.view_method_call((&$contract).contract.$method($($arg),*))
//     };
// }

#[cfg(test)]
mod test {
    use super::Caller;
    use skw_vm_store::create_store;
    
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
                store.clone(), [0u8; 32], AccountId::root(), "dummy".to_string() );

            let _ = caller
                .deploy(
                    [0u8; 300]
                        .as_ref()
                        .into(),
                    AccountId::testn(1),
                    to_yocto("1"),
                );
        

            let _ = caller.create_user(
                AccountId::testn(2),
                to_yocto("200")
            );

            let contract_account = caller.view_account(AccountId::testn(1));
            let normal_account = caller.view_account(AccountId::testn(2));

            assert!(contract_account.is_some());
            assert!(normal_account.is_some());
            caller.write_to_file("./mock/new1");
            caller.state_root()
        };

        {
            let store = create_store();
            store.load_state_from_file("./mock/new1").unwrap();

            let caller = Caller::new(
                store.clone(), state_root, AccountId::system(), 
            "../../skw-contract-sdk/examples/status-message-collections/res/".to_string() );

            let contract_account = caller.view_account(AccountId::testn(1));
            let normal_account = caller.view_account(AccountId::testn(2));
            
            assert!(contract_account.is_some());
            assert!(normal_account.is_some());
        };
    }
}