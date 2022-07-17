use std::collections::HashMap;
use std::sync::Arc;

use crate::{ViewResult, utils::{offchain_id_into_account_id}};

use skw_vm_pool::{types::PoolIterator, TransactionPool};
use skw_vm_primitives::{
    account_id::AccountId,
    account::{AccessKey},
    crypto::{InMemorySigner, KeyType, PublicKey, Signer},
    errors::RuntimeError,
    contract_runtime::{
        CryptoHash, Balance, BlockNumber, Gas, Duration
    },
    profile::ProfileData,
    receipt::Receipt,
    config::RuntimeConfig,
    state_record::{StateRecord},
    transaction::{ExecutionOutcome, ExecutionStatus, SignedTransaction},
    views::ViewApplyState,
};


use skw_vm_primitives::test_utils::account_new;

use skw_vm_runtime::{state_viewer::TrieViewer, ApplyState, Runtime};
use skw_vm_store::{
    create_store, ShardTries, Store, get_access_key,
};

const PALLET_KEY: [u8; 32] = [
    109, 111, 100, 108, 115, 99, 111, 110, 116,
    114,  97,  99,   0,   0,  0,   0,   0,   0,
    0,   0,   0,   0,   0,  0,   0,   0,   0,
    0,   0,   0,   0,   0
];

const DEFAULT_BLOCK_PROD_TIME: Duration = 1_000_000_000;
pub fn init_runtime(
    account_id: AccountId,
    cfg: Option<GenesisConfig>,
    store: Option<&Arc<Store>>,
    state_root: Option<CryptoHash>,
) -> (RuntimeStandalone, InMemorySigner) {
    let mut config = cfg.unwrap_or_default();

    config.runtime_config.wasm_config.limit_config.max_total_prepaid_gas = config.gas_limit;
    let signer = InMemorySigner::from_seed(
        account_id.clone(), KeyType::ED25519, account_id.clone().as_str(),
    );

    // TODO: look deeper into this u128 overflow
    let pallet_root_account = account_new(10u128.pow(30), CryptoHash::default());
    let pallet_root_account_id =  offchain_id_into_account_id(&PALLET_KEY.to_vec());
    let pallet_root_account_signer = InMemorySigner::from_seed(
        pallet_root_account_id.clone(), KeyType::ED25519, &"6d6f646c73636f6e747261630000000000000000000000000000000000000000", 
    );

    config.state_records.push(StateRecord::Account {
        account_id: pallet_root_account_id.clone(),
        account: pallet_root_account,
    });

    config.state_records.push(StateRecord::AccessKey {
        account_id: pallet_root_account_id.clone(),
        public_key: pallet_root_account_signer.public_key(),
        access_key: AccessKey::full_access(),
    });

    let store = match store {
        None => create_store(),
        Some(s) => s.clone(),
    };
    let state_root = match state_root {
        None => CryptoHash::default(),
        Some(sr) => sr
    };

    let runtime = RuntimeStandalone::new(&config, store, state_root);
    (runtime, signer)
}

#[derive(Debug)]
pub struct GenesisConfig {
    pub genesis_time: u64,
    pub gas_price: Balance,
    pub gas_limit: Gas,
    pub genesis_height: u64,
    pub runtime_config: RuntimeConfig,
    pub state_records: Vec<StateRecord>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        let runtime_config = RuntimeConfig::test();

        Self {
            genesis_time: 0,
            gas_price: 100_000_000,
            gas_limit: runtime_config.wasm_config.limit_config.max_total_prepaid_gas,
            genesis_height: 0,
            runtime_config,
            state_records: vec![],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Block {
    prev_block: Option<Arc<Block>>,
    pub state_root: CryptoHash,
    pub block_number: BlockNumber,
    pub block_timestamp: u64,
    pub gas_price: Balance,
    pub gas_limit: Gas,
}

impl Drop for Block {
    fn drop(&mut self) {
        // Blocks form a liked list, so the generated recursive drop overflows
        // the stack. Let's use an explicit loop to avoid that.
        let mut curr = self.prev_block.take();
        while let Some(mut next) = curr.and_then(|it| Arc::try_unwrap(it).ok()) {
            curr = next.prev_block.take();
        }
    }
}

impl Block {
    pub fn genesis(config: &GenesisConfig) -> Self {
        Self {
            prev_block: None,
            state_root: CryptoHash::default(),
            block_number: config.genesis_height,
            block_timestamp: config.genesis_time,
            gas_price: config.gas_price,
            gas_limit: config.gas_limit,
        }
    }

    fn produce(
        &self,
        new_state_root: CryptoHash,
        block_prod_time: Duration,
    ) -> Block {
        Self {
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            block_timestamp: self.block_timestamp + block_prod_time,
            prev_block: Some(Arc::new(self.clone())),
            state_root: new_state_root,
            block_number: self.block_number + 1,
        }
    }
}

pub struct RuntimeStandalone {
    runtime_config: RuntimeConfig,
    tx_pool: TransactionPool,
    transactions: HashMap<CryptoHash, SignedTransaction>,
    outcomes: HashMap<CryptoHash, ExecutionOutcome>,
    profile: HashMap<CryptoHash, ProfileData>,
    pub cur_block: Block,
    runtime: Runtime,
    tries: ShardTries,
    pending_receipts: Vec<Receipt>,
    pub last_outcomes: Vec<CryptoHash>,
}

impl RuntimeStandalone {
    pub fn new(genesis: &GenesisConfig, store: Arc<Store>, state_root: CryptoHash) -> Self {
        let mut genesis_block = Block::genesis(&genesis);
        
        let mut store_update = store.store_update();
        let runtime = Runtime::new();
        let tries = ShardTries::new(store);

        if state_root == CryptoHash::default() {
            let (s_update, state_root) = runtime.apply_genesis_state(
                tries.clone(),
                &genesis.state_records,
                &genesis.runtime_config,
            );

            store_update.merge(s_update);
            store_update.commit().unwrap();

            genesis_block.state_root = state_root;
        } else {
            genesis_block.state_root = state_root;
        }

        Self {
            runtime_config: genesis.runtime_config.clone(),
            tries,
            runtime,
            transactions: HashMap::new(),
            outcomes: HashMap::new(),
            profile: HashMap::new(),
            cur_block: genesis_block,
            tx_pool: TransactionPool::new(None),
            pending_receipts: vec![],
            last_outcomes: vec![],
        }
    }

    /// Processes blocks until the final value is produced
    pub fn resolve_tx(
        &mut self,
        mut tx: SignedTransaction,
    ) -> Result<(CryptoHash, ExecutionOutcome), RuntimeError> {
        tx.init();
        let mut outcome_hash = tx.get_hash();
        self.transactions.insert(outcome_hash, tx.clone());
        self.tx_pool.insert_transaction(tx);
        self.last_outcomes = vec![];
        loop {
            self.produce_block()?;
            if let Some(outcome) = self.outcomes.get(&outcome_hash) {
                match outcome.status {
                    ExecutionStatus::Unknown => unreachable!(), // ExecutionStatus::Unknown is not relevant for a standalone runtime
                    ExecutionStatus::SuccessReceiptId(ref id) => outcome_hash = *id,
                    ExecutionStatus::SuccessValue(_) | ExecutionStatus::Failure(_) => {
                        return Ok((outcome_hash, outcome.clone()))
                    }
                };
            } else if self.pending_receipts.is_empty() {
                unreachable!("Lost an outcome for the receipt hash {:?}", outcome_hash);
            }
        }
    }

    /// Processes all transactions and pending receipts until there is no pending_receipts left
    pub fn process_all(&mut self) -> Result<(), RuntimeError> {
        loop {
            self.produce_block()?;
            if self.pending_receipts.is_empty() {
                return Ok(());
            }
        }
    }

    /// Processes one block. Populates outcomes and producining new pending_receipts.
    pub fn produce_block(&mut self) -> Result<(), RuntimeError> {
        let profile_data = ProfileData::default();
        let apply_state = ApplyState {
            block_number: self.cur_block.block_number,
            prev_block_hash: Default::default(),
            block_hash: Default::default(),
            gas_price: self.cur_block.gas_price,
            block_timestamp: self.cur_block.block_timestamp,
            gas_limit: None,
            random_seed: Default::default(),
            config: Arc::new(self.runtime_config.clone()),
        };

        let apply_result = self.runtime.apply(
            self.tries.get_trie(),
            self.cur_block.state_root,
            &apply_state,
            &self.pending_receipts,
            &Self::prepare_transactions(&mut self.tx_pool),
        )?;

        self.pending_receipts = apply_result.outgoing_receipts;
        apply_result.outcomes.iter().for_each(|outcome| {
            self.last_outcomes.push(outcome.id);
            self.outcomes.insert(outcome.id, outcome.outcome.clone());
            self.profile.insert(outcome.id, profile_data.clone());
        });
        let (update, _) =
            self.tries.apply_all(&apply_result.trie_changes).expect("Unexpected Storage error");
        update.commit().expect("Unexpected io error");
        self.cur_block = self.cur_block.produce(
            apply_result.state_root,
            DEFAULT_BLOCK_PROD_TIME,
        );

        Ok(())
    }

    pub fn view_access_key(&self, account_id: AccountId, public_key: &PublicKey) -> Option<AccessKey> {
        let trie_update = self.tries.new_trie_update(self.cur_block.state_root);
        get_access_key(&trie_update, &account_id, public_key)
            .expect("Unexpected Storage error")
    }

    /// Returns a ViewResult containing the value or error and any logs
    pub fn view_method_call(
        &self,
        account_id: AccountId, 
        function_name: &str,
        args: &[u8],
    ) -> ViewResult {
        let trie_update = self.tries.new_trie_update(self.cur_block.state_root);
        let viewer = TrieViewer::default();
        let mut logs = vec![];
        let view_state = ViewApplyState {
            block_number: self.cur_block.block_number,
            prev_block_hash: CryptoHash::default(), //self.cur_block.prev_block.as_ref().unwrap().state_root,
            block_timestamp: self.cur_block.block_timestamp,
            block_hash: self.cur_block.state_root,
        };
        let result = viewer.call_function(
            trie_update,
            view_state,
            &account_id,
            function_name,
            args,
            &mut logs,
        );
        ViewResult::new(result, logs)
    }

    pub fn state_root(&self) -> CryptoHash {
        self.cur_block.state_root
    }

    fn prepare_transactions(tx_pool: &mut TransactionPool) -> Vec<SignedTransaction> {
        let mut res = vec![];
        let mut pool_iter = tx_pool.pool_iterator();
        while let Some(iter) = pool_iter.next() {
            if let Some(tx) = iter.next() {
                res.push(tx);
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use skw_vm_primitives::account::Account;
    use skw_vm_store::get_account;

    use super::*;
    use crate::utils::to_yocto;

    fn str_to_account_id(s: &str) -> AccountId {
        AccountId::try_from(s.to_string()).unwrap()
    }

    impl RuntimeStandalone {
         /// Just puts tx into the transaction pool
        pub fn send_tx(&mut self, tx: SignedTransaction) -> CryptoHash {
            let tx_hash = tx.get_hash();
            self.transactions.insert(tx_hash, tx.clone());
            self.tx_pool.insert_transaction(tx);
            tx_hash
        }
        
        pub fn outcome(&self, hash: &CryptoHash) -> Option<ExecutionOutcome> {
            self.outcomes.get(hash).cloned()
        }
        
        pub fn produce_blocks(&mut self, num_of_blocks: u64) -> Result<(), RuntimeError> {
            for _ in 0..num_of_blocks {
                self.produce_block()?;
            }
            Ok(())
        }

        pub fn view_account(&self, account_id: AccountId) -> Option<Account> {
            let trie_update = self.tries.new_trie_update(self.cur_block.state_root);
            get_account(&trie_update, &account_id).expect("Unexpected Storage error")
        }

    }

    #[test]
    fn single_block() {
        let root = str_to_account_id(&"modlscontrac");

        let (mut runtime, signer) = init_runtime(root, None, None, None);
        let hash = runtime.send_tx(SignedTransaction::create_account(
            1,
            signer.account_id.clone(),
            str_to_account_id(&"bob"),
            100,
            signer.public_key(),
            &signer,
            CryptoHash::default(),
        ));
        runtime.produce_block().unwrap();
        assert!(matches!(
            runtime.outcome(&hash),
            Some(ExecutionOutcome { status: ExecutionStatus::SuccessReceiptId(_), .. })
        ));
    }

    #[test]
    fn process_all() {
        let root = str_to_account_id(&"modlscontrac");

        let (mut runtime, signer) = init_runtime(root, None, None, None);
        assert_eq!(runtime.view_account(str_to_account_id(&"bob")), None);
        let outcome = runtime.resolve_tx(SignedTransaction::create_account(
            1,
            str_to_account_id(&"modlscontrac"),
            str_to_account_id(&"bob"),
            165437999999999999999000,
            signer.public_key(),
            &signer,
            CryptoHash::default(),
        ));
        assert!(matches!(
            outcome,
            Ok((_, ExecutionOutcome { status: ExecutionStatus::SuccessValue(_), .. }))
        ));
        assert_eq!(runtime.view_account(str_to_account_id(&"bob")).unwrap().amount(), 165437999999999999999000);
        assert_eq!(runtime.view_account(str_to_account_id(&"bob")).unwrap().code_hash(), CryptoHash::default());
        assert_eq!(runtime.view_account(str_to_account_id(&"bob")).unwrap().locked(), 0);
        assert_eq!(runtime.view_account(str_to_account_id(&"bob")).unwrap().storage_usage(), 182);
    }

    #[test]
    fn test_cross_contract_call() {
        let root = str_to_account_id(&"modlscontrac");
        let (mut runtime, signer) = init_runtime(root, None, None, None);
        assert!(matches!(
            runtime.resolve_tx(SignedTransaction::create_contract(
                1,
                signer.account_id.clone(),
                str_to_account_id(&"status"),
                include_bytes!("../../skw-contract-sdk/examples/status-message/res/status_message.wasm")
                    .as_ref()
                    .into(),
                to_yocto("35"),
                signer.public_key(),
                &signer,
                CryptoHash::default(),
            )),
            Ok((_, ExecutionOutcome { status: ExecutionStatus::SuccessValue(_), .. }))
        ));
        let res = runtime.resolve_tx(SignedTransaction::create_contract(
            2,
            signer.account_id.clone(),
            str_to_account_id(&"caller"),
            include_bytes!(
                "../../skw-contract-sdk/examples/cross-contract-high-level/res/cross_contract_high_level.wasm"
            )
            .as_ref()
            .into(),
            to_yocto("35"),
            signer.public_key(),
            &signer,
            CryptoHash::default(),
        ));
        assert!(matches!(
            res,
            Ok((_, ExecutionOutcome { status: ExecutionStatus::SuccessValue(_), .. }))
        ));
        let res = runtime.resolve_tx(SignedTransaction::call(
            3,
            signer.account_id.clone(),
            str_to_account_id(&"caller"),
            &signer,
            0,
            "simple_call".into(),
            "{\"account_id\": \"status\", \"message\": \"caller status is ok!\"}"
                .as_bytes()
                .to_vec(),
            300_000_000_000_000,
            CryptoHash::default(),
        ));
        let (_, res) = res.unwrap();
        runtime.process_all().unwrap();

        assert!(matches!(res, ExecutionOutcome { status: ExecutionStatus::SuccessValue(_), .. }));
        let res = runtime.view_method_call(str_to_account_id(&"status"), "get_status", b"{\"account_id\": \"modlscontrac\"}");

        let parsed_res = res.result();

        println!("{:?}", res);

        let caller_status = String::from_utf8(parsed_res.0.unwrap()).unwrap();
        assert_eq!("\"caller status is ok!\"", caller_status);
    }

    #[test]
    fn can_produce_many_blocks_without_stack_overflow() {
        let root = str_to_account_id(&"modlscontrac");
        let (mut runtime, _) = init_runtime(root, None, None, None);
        runtime.produce_blocks(20_000).unwrap();
    }
}
