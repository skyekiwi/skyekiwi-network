//! Method with non-deserializable argument type.

use borsh::{BorshDeserialize, BorshSerialize};
use skw_contract_sdk::{skw_bindgen, PanicOnDefault};

#[skw_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
struct Storage {}

#[skw_bindgen]
impl Storage {
    pub fn insert(&mut self, (a, b): (u8, u32)) {}
}

fn main() {}
