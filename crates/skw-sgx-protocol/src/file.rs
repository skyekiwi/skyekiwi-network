// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
use sgx_tstd::{vec::Vec, io::{Read, Write}};
use libflate::zlib::{Encoder, Decoder};
use hmac_sha256::Hash;

pub struct FileHandle();

impl FileHandle {

	pub fn deflate_chunk(content: Vec<u8>) -> Vec<u8> {
		let mut encoder = Encoder::new(Vec::new()).unwrap();
		encoder.write_all(&content).unwrap();
    	encoder.finish().into_result().unwrap()
	}

	pub fn inflate_chunk(deflated_content: Vec<u8>) -> Vec<u8> {
		let mut decoder = Decoder::new(&deflated_content[..]).unwrap();
		let mut decoded_data = Vec::new();
		decoder.read_to_end(&mut decoded_data).unwrap();
		decoded_data.to_vec()
	}

	pub fn sha256_checksum(chunk: Vec<u8>) -> crate::types::file::Hash {
		Hash::hash(&chunk[..])
	}
}
