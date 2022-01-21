//! Rust contract that uses conditional compilation.

use skw_contract_sdk::skw_bindgen;
use borsh::{BorshDeserialize, BorshSerialize};

#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
struct Incrementer {
    value: u32,
}

#[skw_bindgen(init => new)]
impl Incrementer {
    #[cfg(feature = "myfeature")]
    pub fn new() -> Self {
        Self {value: 0}
    }

    #[cfg(not(feature = "myfeature"))]
    pub fn new() -> Self {
        Self {value: 1}
    }

    #[cfg(feature = "myfeature")]
    pub fn inc(&mut self, by: u32) {
        self.value += by;
    }

    #[cfg(not(feature = "myfeature"))]
    pub fn inc(&mut self, by: u32) {
        self.value += by;
    }
}

fn main() {}
