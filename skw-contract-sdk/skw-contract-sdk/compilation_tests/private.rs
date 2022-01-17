//! Regular smart contract.

use borsh::{BorshDeserialize, BorshSerialize};
use skw_contract_sdk::skw_bindgen;

#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
struct Incrementer {
    value: u32,
}

#[skw_bindgen]
impl Incrementer {
    #[private]
    pub fn inc(&mut self, by: u32) {
        self.value += by;
    }
}

fn main() {}
