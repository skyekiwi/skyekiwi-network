//! Smart contract with initialization function.

use skw_contract_sdk::skw_bindgen;
use borsh::{BorshDeserialize, BorshSerialize};

#[skw_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
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
        Self {
            value: starting_value
        }
    }
}

fn main() {}
