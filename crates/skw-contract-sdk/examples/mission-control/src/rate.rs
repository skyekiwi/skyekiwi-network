use crate::account::*;
use crate::asset::*;
use skw_contract_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use skw_contract_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "skw_contract_sdk::serde")]
pub struct Rate {
    pub credit: HashMap<Asset, Quantity>,
    pub debit: HashMap<Asset, Quantity>,
}

impl Rate {}
