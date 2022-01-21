// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
use std::{vec::Vec, num::ParseIntError, string::String, fmt::Write};

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
