use skw_contract_sdk::{skw_bindgen, metadata};
use borsh::{BorshDeserialize, BorshSerialize};
metadata! {
FOOBAR

#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
struct Incrementer {
    value: u32,
}

#[skw_bindgen]
impl Incrementer {
    pub fn inc(&mut self, by: u32) {
        self.value += by;
    }
}
}

fn main() {}
