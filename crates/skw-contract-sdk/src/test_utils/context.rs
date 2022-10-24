use crate::mock::MockedBlockchain;
use crate::{Balance, BlockNumber, Gas, StorageUsage, AccountId, PromiseResult, RuntimeFeesConfig};

use skw_vm_host::{VMConfig, ViewConfig, VMContext};
use std::convert::TryInto;

/// Simple VMContext builder that allows to quickly create custom context in tests.
#[derive(Clone)]
pub struct VMContextBuilder {
    pub context: VMContext,
}

impl Default for VMContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl VMContextBuilder {
    pub fn new() -> Self {
        Self {
            context: VMContext {
                current_account_id: AccountId::test(0).into(),
                signer_account_id: AccountId::test(2).into(),
                predecessor_account_id: AccountId::test(2).into(),
                input: vec![],
                block_number: 0,
                block_timestamp: 0,
                account_balance: 10u128.pow(26),
                storage_usage: 1024 * 300,
                attached_deposit: 0,
                prepaid_gas: 300 * 10u64.pow(12),
                random_seed: vec![0u8; 32],
                view_config: None,
                output_data_receivers: vec![],
            },
        }
    }

    pub fn current_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.context.current_account_id = account_id.try_into().unwrap();
        self
    }

    pub fn signer_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.context.signer_account_id = account_id.try_into().unwrap();
        self
    }

    pub fn predecessor_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.context.predecessor_account_id = account_id.try_into().unwrap();
        self
    }

    pub fn block_number(&mut self, block_number: BlockNumber) -> &mut Self {
        self.context.block_number = block_number;
        self
    }

    pub fn block_timestamp(&mut self, block_timestamp: u64) -> &mut Self {
        self.context.block_timestamp = block_timestamp;
        self
    }

    pub fn account_balance(&mut self, amount: Balance) -> &mut Self {
        self.context.account_balance = amount;
        self
    }

    pub fn storage_usage(&mut self, usage: StorageUsage) -> &mut Self {
        self.context.storage_usage = usage;
        self
    }

    pub fn attached_deposit(&mut self, amount: Balance) -> &mut Self {
        self.context.attached_deposit = amount;
        self
    }

    pub fn prepaid_gas(&mut self, gas: Gas) -> &mut Self {
        self.context.prepaid_gas = gas;
        self
    }

    pub fn random_seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.context.random_seed = seed.to_vec();
        self
    }

    pub fn is_view(&mut self, is_view: bool) -> &mut Self {
        self.context.view_config =
            if is_view { Some(ViewConfig { max_gas_burnt: 200000000000000 }) } else { None };
        self
    }

    pub fn build(&self) -> VMContext {
        self.context.clone()
    }
}

/// Initializes the [`MockedBlockchain`] with a single promise result during execution.
#[deprecated(since = "4.0.0", note = "Use `testing_env!` macro to initialize with promise results")]
pub fn testing_env_with_promise_results(context: VMContext, promise_result: PromiseResult) {
    let storage = crate::mock::with_mocked_blockchain(|b| b.take_storage());

    //? This probably shouldn't need to replace the existing mocked blockchain altogether?
    //? Might be a good time to remove this utility function altogether
    crate::env::set_blockchain_interface(MockedBlockchain::new(
        context,
        VMConfig::test(),
        RuntimeFeesConfig::test(),
        vec![promise_result],
        storage,
        Default::default(),
        None,
    ));
}
