// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
	vec::Vec, 
	num::ParseIntError, 
	string::String, 
	fmt::Write,
	format
};

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}

pub fn encode_hex(bytes: &[u8]) -> Result<String, ()> {
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		write!(&mut s, "{:02x}", b).unwrap();
	}
	Ok(s)
}

pub fn pad_usize(size: usize) -> Vec<u8> {
	let x = format!("{:0>8x}", size);
	decode_hex(&x).unwrap()
}

pub fn padded_slice_to_usize(x: &[u8]) -> usize {
	let mut result: usize = 0;
	let mut m = 1;
	for i in x.iter().rev() {
		result += (*i as usize) * m;
		m *= 0x100;
	}
	result
}
