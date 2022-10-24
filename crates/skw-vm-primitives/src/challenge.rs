use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize};

pub type StateItem = Vec<u8>;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PartialState(pub Vec<StateItem>);
