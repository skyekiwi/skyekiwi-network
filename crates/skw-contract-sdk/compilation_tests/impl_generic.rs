//! Impl block has type parameters.

use borsh::{BorshDeserialize, BorshSerialize};
use skw_contract_sdk::skw_bindgen;
use std::marker::PhantomData;

#[skw_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
struct Incrementer<T> {
    value: u32,
    data: PhantomData<T>,
}

#[skw_bindgen]
impl<'a, T: 'a + std::fmt::Display> Incrementer<T> {
    pub fn inc(&mut self, by: u32) {
        self.value += by;
    }
}

fn main() {}
