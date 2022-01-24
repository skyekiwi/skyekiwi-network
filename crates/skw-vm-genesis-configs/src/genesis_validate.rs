use std::collections::{HashMap, HashSet};

use crate::genesis_config::{Genesis, GenesisConfig};
use skw_vm_primitives::state_record::StateRecord;
use skw_vm_primitives::contract_runtime::AccountId;
use num_rational::Rational;

/// Validate genesis config and records. Panics if genesis is ill-formed.
pub fn validate_genesis(genesis: &Genesis) {
    let mut genesis_validator = GenesisValidator::new(&genesis.config);
    genesis.for_each_record(|record: &StateRecord| {
        genesis_validator.process_record(record);
    });
    genesis_validator.validate();
}

struct GenesisValidator<'a> {
    genesis_config: &'a GenesisConfig,
    total_supply: u128,
    account_ids: HashSet<AccountId>,
    contract_account_ids: HashSet<AccountId>,
}

impl<'a> GenesisValidator<'a> {
    pub fn new(genesis_config: &'a GenesisConfig) -> Self {
        Self {
            genesis_config,
            total_supply: 0,
            account_ids: HashSet::new(),
            contract_account_ids: HashSet::new(),
        }
    }

    pub fn process_record(&mut self, record: &StateRecord) {
        match record {
            StateRecord::Account { account_id, account } => {
                if self.account_ids.contains(account_id) {
                    panic!("Duplicate account id {} in genesis records", account_id);
                }
                self.total_supply += account.locked() + account.amount();
                self.account_ids.insert(account_id.clone());
            }
            StateRecord::Contract { account_id, .. } => {
                if self.contract_account_ids.contains(account_id) {
                    panic!("account {} has more than one contract deployed", account_id);
                }
                self.contract_account_ids.insert(account_id.clone());
            }
            _ => {}
        }
    }

    pub fn validate(&self) {
        for account_id in &self.contract_account_ids {
            assert!(
                self.account_ids.contains(account_id),
                "contract account {} does not exist",
                account_id
            );
        }
        assert_eq!(self.total_supply, self.genesis_config.total_supply, "wrong total supply");
        assert!(
            self.genesis_config.gas_price_adjustment_rate < Rational::from_integer(1),
            "Gas price adjustment rate must be less than 1"
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::GenesisRecords;
    use near_crypto::{KeyType, PublicKey};
    use skw_vm_primitives::account::{Account};
    use skw_vm_primitives::contract_runtime::AccountInfo;

    const VALID_ED25519_RISTRETTO_KEY: &str = "ed25519:KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7";

    fn create_account() -> Account {
        Account::new(100, 10, Default::default(), 0)
    }

    #[test]
    #[should_panic(expected = "wrong total supply")]
    fn test_total_supply_not_match() {
        let mut genesis = Genesis::default();
        genesis.records = GenesisRecords(vec![StateRecord::Account {
            account_id: "test".parse().unwrap(),
            account: create_account(),
        }]);
        validate_genesis(&genesis);
    }

    #[test]
    #[should_panic(expected = "account test has more than one contract deployed")]
    fn test_more_than_one_contract() {
        let mut genesis = Genesis::default();
        genesis.config.total_supply = 110;
        genesis.records = GenesisRecords(vec![
            StateRecord::Account { account_id: "test".parse().unwrap(), account: create_account() },
            StateRecord::Contract { account_id: "test".parse().unwrap(), code: [1, 2, 3].to_vec() },
            StateRecord::Contract {
                account_id: "test".parse().unwrap(),
                code: [1, 2, 3, 4].to_vec(),
            },
        ]);
        validate_genesis(&genesis);
    }
}
