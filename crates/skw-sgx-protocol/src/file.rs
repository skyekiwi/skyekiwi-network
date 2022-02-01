// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
use sgx_tstd::{vec::Vec, io::{Read, Write}, path::PathBuf};
use libflate::zlib::{Encoder, Decoder};
use std::SgxFile;
use crate::types::{
	file::{FileHandleError, ReadOutput},
}

pub struct FileHandle(path: PathBuf);

impl FileHandle {

	// this is only used for testing
	pub fn read_to_end(&self, buf: &[u8]) -> Result<usize, FileHandleError> {
		let len = SgxFile::read(self.0, buf).map_err(|_| FileHandleError::FileNotFound)?;
		Ok( len )
	}

	pub fn read_exact(&self, buf: &[u8]) -> Result<usize, FileHandleError> {
		let mut file = SgxFile::open(&self.0);
		let len = file.read_exact(buf).map_err(|_| FileHandleError::FileNotFound)?;
		Ok( len )
	}

	pub fn write(&self, buf: &[u8]) -> Result<usize, FileHandleError> {
		let mut file = SgxFile::create(&self.0);
		let len = file.write(buf).map_err(|_| FileHandleError::FileNotFound)?;
		Ok( len )
	}

	pub fn deflate_chunk(content: &[u8]) -> Vec<u8> {
		let mut encoder = Encoder::new(Vec::new()).unwrap();
		encoder.write_all(&content).unwrap();
    	encoder.finish().into_result().unwrap()
	}

	pub fn inflate_chunk(deflated_content: &[u8]) -> Vec<u8> {
		let mut decoder = Decoder::new(&deflated_content[..]).unwrap();
		let mut decoded_data = Vec::new();
		decoder.read_to_end(&mut decoded_data).unwrap();
		decoded_data.to_vec()
	}

	pub fn sha256_checksum(chunk: &[u8]) -> crate::types::file::Hash {
		hmac_sha256::Hash::hash(&chunk[..])
	}
}
