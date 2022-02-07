use std::slice;
use std::fs::File;
use std::convert::TryInto;
use std::path::PathBuf;
use std::string::String;
use std::io::Read;
use std::num::ParseIntError;
use std::io::Write;

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

pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		write!(&mut s, "{:02x}", b).unwrap();
	}
	s
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}