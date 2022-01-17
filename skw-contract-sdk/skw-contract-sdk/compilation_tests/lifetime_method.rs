//! Method signature uses lifetime.

use skw_contract_sdk::skw_bindgen;
use borsh::{BorshDeserialize, BorshSerialize};

#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
struct Ident {
    value: u32,
}

#[skw_bindgen]
impl Ident {
    pub fn is_ident<'a>(&self, other: &'a u32) -> Option<&'a u32> {
        if *other == self.value {
            Some(other)
        } else {
            None
        }
    }
}

fn main() {}
