use std::convert::TryFrom;

use skw_vm_primitives::account_id::AccountId;

#[cfg(test)]
pub fn to_yocto(value: &str) -> u128 {
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
        part1 + part2
    } else {
        part1
    }
}

pub fn str_to_account_id(s: &str) -> AccountId {
    AccountId::try_from(s.to_string()).unwrap()
}

pub fn offchain_id_into_account_id(id: &Vec<u8>) -> AccountId {
    let account_id_str = hex::encode(id);
    AccountId::try_from(account_id_str).unwrap()
}

pub fn vec_to_str(buf: &Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}
