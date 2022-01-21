//! Payable view are not valid

use borsh::{BorshDeserialize, BorshSerialize};
use skw_contract_sdk::skw_bindgen;

#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
struct Test {}

#[skw_bindgen]
impl Test {
    #[payable]
    pub fn pay(&self) {}
}

fn main() {}
