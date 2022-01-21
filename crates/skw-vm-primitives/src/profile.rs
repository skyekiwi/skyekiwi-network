use std::fmt;
use std::ops::{Index, IndexMut};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::config::{ActionCosts, ExtCosts};

#[derive(Clone, PartialEq, Eq)]
pub struct DataArray(Box<[u64; Self::LEN]>);

impl DataArray {
    pub const LEN: usize = Cost::ALL.len();
}

impl Index<usize> for DataArray {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for DataArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl BorshDeserialize for DataArray {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        let data_vec: Vec<u64> = BorshDeserialize::deserialize(buf)?;
        let mut data_array = [0; Self::LEN];
        let len = Self::LEN.min(data_vec.len());
        data_array[0..len].copy_from_slice(&data_vec[0..len]);
        Ok(Self(Box::new(data_array)))
    }
}

impl BorshSerialize for DataArray {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        let v: Vec<_> = self.0.as_ref().to_vec();
        BorshSerialize::serialize(&v, writer)
    }
}

/// Profile of gas consumption.
/// When add new cost, the new cost should also be append to Cost::ALL
#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Eq)]
pub struct ProfileData {
    data: DataArray,
}

impl Default for ProfileData {
    fn default() -> ProfileData {
        ProfileData::new()
    }
}

impl ProfileData {
    #[inline]
    pub fn new() -> Self {
        let costs = DataArray(Box::new([0; DataArray::LEN]));
        ProfileData { data: costs }
    }

    #[inline]
    pub fn merge(&mut self, other: &ProfileData) {
        for i in 0..DataArray::LEN {
            self.data[i] = self.data[i].saturating_add(other.data[i]);
        }
    }

    #[inline]
    pub fn add_action_cost(&mut self, action: ActionCosts, value: u64) {
        self[Cost::ActionCost { action_cost_kind: action }] =
            self[Cost::ActionCost { action_cost_kind: action }].saturating_add(value);
    }

    #[inline]
    pub fn add_ext_cost(&mut self, ext: ExtCosts, value: u64) {
        self[Cost::ExtCost { ext_cost_kind: ext }] =
            self[Cost::ExtCost { ext_cost_kind: ext }].saturating_add(value);
    }

    /// WasmInstruction is the only cost we don't explicitly account for.
    /// Instead, we compute it at the end of contract call as the difference
    /// between total gas burnt and what we've explicitly accounted for in the
    /// profile.
    ///
    /// This is because WasmInstruction is the hottest cost and is implemented
    /// with the help on the VM side, so we don't want to have profiling logic
    /// there both for simplicity and efficiency reasons.
    pub fn compute_wasm_instruction_cost(&mut self, total_gas_burnt: u64) {
        let mut value = total_gas_burnt;
        for cost in Cost::ALL {
            value = value.saturating_sub(self[*cost]);
        }
        self[Cost::WasmInstruction] = value
    }

    pub fn get_action_cost(&self, action: ActionCosts) -> u64 {
        self[Cost::ActionCost { action_cost_kind: action }]
    }

    pub fn get_ext_cost(&self, ext: ExtCosts) -> u64 {
        self[Cost::ExtCost { ext_cost_kind: ext }]
    }

    pub fn host_gas(&self) -> u64 {
        let mut host_gas = 0u64;
        for cost in Cost::ALL {
            match cost {
                Cost::ExtCost { ext_cost_kind: e } => host_gas += self.get_ext_cost(*e),
                _ => {}
            }
        }
        host_gas
    }

    pub fn action_gas(&self) -> u64 {
        let mut action_gas = 0u64;
        for cost in Cost::ALL {
            match cost {
                Cost::ActionCost { action_cost_kind: a } => action_gas += self.get_action_cost(*a),
                _ => {}
            }
        }
        action_gas
    }
}

impl fmt::Debug for ProfileData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use num_rational::Ratio;
        let host_gas = self.host_gas();
        let action_gas = self.action_gas();

        writeln!(f, "------------------------------")?;
        writeln!(f, "Action gas: {}", action_gas)?;
        writeln!(f, "------ Host functions --------")?;
        for cost in Cost::ALL {
            match cost {
                Cost::ExtCost { ext_cost_kind: e } => {
                    let d = self.get_ext_cost(*e);
                    if d != 0 {
                        writeln!(
                            f,
                            "{} -> {} [{}% host]",
                            ExtCosts::name_of(*e as usize),
                            d,
                            Ratio::new(d * 100, core::cmp::max(host_gas, 1)).to_integer(),
                        )?;
                    }
                }
                _ => {}
            }
        }
        writeln!(f, "------ Actions --------")?;
        for cost in Cost::ALL {
            match cost {
                Cost::ActionCost { action_cost_kind: a } => {
                    let d = self.get_action_cost(*a);
                    if d != 0 {
                        writeln!(f, "{} -> {}", ActionCosts::name_of(*a as usize), d)?;
                    }
                }
                _ => {}
            }
        }
        writeln!(f, "------------------------------")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Cost {
    ActionCost { action_cost_kind: ActionCosts },
    ExtCost { ext_cost_kind: ExtCosts },
    WasmInstruction,
}

impl Cost {
    pub const ALL: &'static [Cost] = &[
        // ActionCost is unlikely to have new ones, so have it at first
        Cost::ActionCost { action_cost_kind: ActionCosts::deploy_contract },
        Cost::ActionCost { action_cost_kind: ActionCosts::function_call },
        Cost::ActionCost { action_cost_kind: ActionCosts::value_return },
        Cost::ActionCost { action_cost_kind: ActionCosts::new_receipt },
        Cost::ExtCost { ext_cost_kind: ExtCosts::base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_bytes },
        Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_block },
        Cost::ExtCost { ext_cost_kind: ExtCosts::ecrecover_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::log_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::log_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_key_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_value_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_evicted_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_key_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_value_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_key_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_ret_value_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_byte },
        Cost::ExtCost { ext_cost_kind: ExtCosts::touching_trie_node },
        Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_base },
        Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_per_promise },
        Cost::ExtCost { ext_cost_kind: ExtCosts::promise_return },
        Cost::WasmInstruction,
    ];

    pub fn index(self) -> usize {
        match self {
            Cost::ActionCost { action_cost_kind: ActionCosts::deploy_contract } => 0,
            Cost::ActionCost { action_cost_kind: ActionCosts::function_call } => 1,
            Cost::ActionCost { action_cost_kind: ActionCosts::value_return } => 2,
            Cost::ActionCost { action_cost_kind: ActionCosts::new_receipt } => 3,
            Cost::ActionCost { action_cost_kind: ActionCosts::__count } => unreachable!(),
            Cost::ExtCost { ext_cost_kind: ExtCosts::base } => 4,
            Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_base } => 5,
            Cost::ExtCost { ext_cost_kind: ExtCosts::contract_compile_bytes } => 6,
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_base } => 7,
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_memory_byte } => 8,
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_base } => 9,
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_memory_byte } => 10,
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_base } => 11,
            Cost::ExtCost { ext_cost_kind: ExtCosts::read_register_byte } => 12,
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_base } => 13,
            Cost::ExtCost { ext_cost_kind: ExtCosts::write_register_byte } => 14,
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_base } => 15,
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf8_decoding_byte } => 16,
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_base } => 17,
            Cost::ExtCost { ext_cost_kind: ExtCosts::utf16_decoding_byte } => 18,
            Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_base } => 19,
            Cost::ExtCost { ext_cost_kind: ExtCosts::sha256_byte } => 20,
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_base } => 21,
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak256_byte } => 22,
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_base } => 23,
            Cost::ExtCost { ext_cost_kind: ExtCosts::keccak512_byte } => 24,
            Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_base } => 25,
            Cost::ExtCost { ext_cost_kind: ExtCosts::ripemd160_block } => 26,
            Cost::ExtCost { ext_cost_kind: ExtCosts::ecrecover_base } => 27,
            Cost::ExtCost { ext_cost_kind: ExtCosts::log_base } => 28,
            Cost::ExtCost { ext_cost_kind: ExtCosts::log_byte } => 29,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_base } => 30,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_key_byte } => 31,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_value_byte } => 32,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_write_evicted_byte } => 33,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_base } => 34,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_key_byte } => 35,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_read_value_byte } => 36,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_base } => 37,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_key_byte } => 38,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_remove_ret_value_byte } => 39,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_base } => 40,
            Cost::ExtCost { ext_cost_kind: ExtCosts::storage_has_key_byte } => 41,
            Cost::ExtCost { ext_cost_kind: ExtCosts::touching_trie_node } => 42,
            Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_base } => 43,
            Cost::ExtCost { ext_cost_kind: ExtCosts::promise_and_per_promise } => 44,
            Cost::ExtCost { ext_cost_kind: ExtCosts::promise_return } => 45,
            Cost::WasmInstruction => 46,
            Cost::ExtCost { ext_cost_kind: ExtCosts::__count } => unreachable!(),
        }
    }
}

impl Index<Cost> for ProfileData {
    type Output = u64;

    fn index(&self, index: Cost) -> &Self::Output {
        &self.data[index.index()]
    }
}

impl IndexMut<Cost> for ProfileData {
    fn index_mut(&mut self, index: Cost) -> &mut Self::Output {
        &mut self.data[index.index()]
    }
}
