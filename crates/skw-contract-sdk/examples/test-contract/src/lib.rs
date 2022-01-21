use skw_contract_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use skw_contract_sdk::{env, skw_bindgen};

#[skw_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TestContract {}

impl Default for TestContract {
    fn default() -> Self {
        Self {}
    }
}

#[skw_bindgen]
impl TestContract {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        #[derive(BorshDeserialize)]
        struct OldContract {
            // ...
        }

        let _old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {}
    }

    pub fn test_panic_macro(&mut self) {
        panic!("PANIC!");
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "PANIC!")]
    fn test_panic() {
        let mut contract = TestContract::new();
        contract.test_panic_macro();
    }
}
