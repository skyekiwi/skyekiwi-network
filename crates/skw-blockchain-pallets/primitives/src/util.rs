use sp_std::vec::Vec;

use crate::types::{PublicKey, Bytes};

pub fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| s[i] * 16 + s[i + 1])
        .collect()
}
pub fn decode_hex(s: &str) -> Vec<u8> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
		.collect()
}

pub fn pad_size(size: usize) -> [u8; 4] {
    let mut v = [0, 0, 0, 0];
    v[3] = (size & 0xff) as u8;
    v[2] = ((size >> 8) & 0xff) as u8;
    v[1] = ((size >> 16) & 0xff) as u8;
    v[0] = ((size >> 24) & 0xff) as u8;
    v
}

pub fn unpad_size(size: &[u8; 4]) -> usize {
    if size.len() != 4 {
        panic!("Invalid size");
    }
    return (
        size[3] as usize | 
        ((size[2] as usize) << 8) | 
        ((size[1] as usize) << 16) | 
        ((size[0] as usize) << 24)
    ).into();
}

pub fn public_key_to_offchain_id(pk: &PublicKey) -> Bytes {
    // v1 simply return the pk
    pk.to_vec()
}