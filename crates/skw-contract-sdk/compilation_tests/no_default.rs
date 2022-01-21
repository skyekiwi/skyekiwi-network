//! Smart contract with initialization function.

use borsh::{BorshDeserialize, BorshSerialize};
use skw_contract_sdk::{skw_bindgen, PanicOnDefault};

#[skw_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
struct Incrementer {
    value: u32,
}

#[skw_bindgen]
impl Incrementer {
    pub fn inc(&mut self, by: u32) {
        self.value += by;
    }
    #[init]
    pub fn new(starting_value: u32) -> Self {
        Self { value: starting_value }
    }
}

fn main() {}
