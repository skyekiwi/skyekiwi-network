use skw_vm_primitives::config::ViewConfig;
use skw_vm_host::{VMContext, VMLimitConfig};

#[allow(dead_code)]
pub fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
    VMContext {
        current_account_id: "alice.near".parse().unwrap(),
        signer_account_id: "bob.near".parse().unwrap(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "carol.near".parse().unwrap(),
        input,
        block_number: 0,
        block_timestamp: 0,
        account_balance: 100,
        storage_usage: 0,
        attached_deposit: 10,
        prepaid_gas: 10_u64.pow(14),
        random_seed: vec![],
        view_config: match is_view {
            true => Some(ViewConfig { max_gas_burnt: VMLimitConfig::test().max_gas_burnt }),
            false => None,
        },
        output_data_receivers: vec![],
    }
}
