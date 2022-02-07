// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use sgx_tstd::{
	vec::Vec, 
	io::{Read, Write}, 
	path::PathBuf,
	sgxfs::SgxFile,
};
use libflate::zlib::{Encoder, Decoder};
use crate::types::{
	file::FileError,
};

pub struct FileHandle();

impl FileHandle {
	pub fn read<R>(reader: R, buf: &mut [u8], limit: u64) -> Result<usize, FileError>
	where 
		R:Read
	{
		let mut chunk = reader.take(limit);
		let n = chunk.read_to_end(&mut buf.to_vec());
		match n {
			Err(_) => Err(FileError::FileNotFound),
			Ok(n) => Ok(n),
		}
	}

	pub fn read_all(path: &PathBuf) -> Result<Vec<u8>, FileError> {
		std::sgxfs::read(path).map_err(|_| FileError::FileNotFound)
	}

	pub fn write(path: &PathBuf, buf: &[u8]) -> Result<usize, FileError> {
		let mut file = SgxFile::create(&path).map_err(|_| FileError::FileNotFound)?;
		file.write_all(buf).map_err(|_| FileError::FileNotFound)?;
		
		// TODO: handle this 
		file.flush();
		Ok( buf.len() )
	}

	pub fn unstrusted_write(path: &PathBuf, buf: &[u8], append: bool) -> Result<usize, FileError> {
		let mut file = std::untrusted::fs::OpenOptions::new()
			.write(true)
			.append(append)
			.create(true)
			.open(&path).map_err(|_| FileError::FileNotFound)?;

		file.write_all(buf).map_err(|_| FileError::FileNotFound)?;
		
		// TODO: handle this 
		file.flush();
		Ok( buf.len() )
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
