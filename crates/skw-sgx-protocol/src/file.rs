// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
use sgx_tstd::{vec::Vec, io::{Read, Write}, path::PathBuf};
use libflate::zlib::{Encoder, Decoder};
use std::sgxfs::SgxFile;
use crate::types::{
	file::{FileError, ReadOutput},
};

pub struct FileHandle{
	path: PathBuf
}

impl FileHandle {

	pub fn new(path: PathBuf) -> Self {
		FileHandle {
			path
		}
	}

	// // this is only used for testing
	// pub fn read_to_end(&self, buf: &[u8]) -> Result<usize, FileError> {
	// 	let len = SgxFile::read(self.path, buf).map_err(|_| FileError::FileNotFound)?;
	// 	Ok( len )
	// }

	pub fn read(&self, buf: &mut [u8]) -> Result<usize, FileError> {
		let mut file = SgxFile::open(&self.path).map_err(|_| FileError::FileNotFound)?;
		let len = file.read(buf).map_err(|_| FileError::FileNotFound)?;
		Ok( len )
	}

	pub fn write(&self, buf: &[u8]) -> Result<usize, FileError> {
		let mut file = SgxFile::create(&self.path).map_err(|_| FileError::FileNotFound)?;
		let len = file.write(buf).map_err(|_| FileError::FileNotFound)?;
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
