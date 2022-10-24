use skw_contract_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use skw_contract_sdk::{env, log, metadata, skw_bindgen, AccountId};

use std::collections::HashMap;

metadata! {
#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StatusMessage {
    records: HashMap<AccountId, String>,
}

#[skw_bindgen]
impl StatusMessage {
    #[payable]
    pub fn set_status(&mut self, message: String) {
        let account_id = env::signer_account_id();
        log!("{} set_status with message {}", account_id, message);
        self.records.insert(account_id, message);
    }

    pub fn get_status(&self, account_id: AccountId) -> Option::<String> {
        log!("get_status for account_id {}", account_id);
        self.records.get(&account_id).cloned()
    }
}
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use skw_contract_sdk::test_utils::{get_logs, VMContextBuilder};
    use skw_contract_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(AccountId::test(1))
            .is_view(is_view)
            .build()
    }

    #[test]
    fn set_get_message() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = StatusMessage::default();
        
        contract.set_status("hello".to_string());
        assert_eq!(get_logs(), vec!["sr25519:4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi set_status with message hello"]);
        
        let context = get_context(true);
        testing_env!(context);
        assert_eq!("hello".to_string(), contract.get_status(AccountId::test(1)).unwrap());
        assert_eq!(get_logs(), vec!["get_status for account_id sr25519:4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi"])
    }

    #[test]
    fn get_nonexistent_message() {
        let context = get_context(true);
        testing_env!(context);
        let contract = StatusMessage::default();
        assert_eq!(None, contract.get_status(AccountId::test(2)));
        assert_eq!(get_logs(), vec!["get_status for account_id sr25519:8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR"])
    }
}
