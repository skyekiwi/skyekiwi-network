// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{vec::Vec, convert::TryInto};
use std::path::PathBuf;
use std::collections::HashMap;
use std::string::String;

use crate::types::{
	metadata::*,
	crypto::{BoxSecretKey, BoxCipher, BoxPublicKey, CryptoError, SecretboxCipher, SecretboxKey},
	file::Hash,
};
use crate::crypto::NaClBox;
use crate::file::FileHandle;

fn validate_pre_seal(_pre_seal: &PreSeal) -> Result<(), MetadataError> {
	Ok(())
}
fn validate_sealed_metadata(_sealed: &SealedMetadata) -> Result<(), MetadataError> {
	Ok(())
}

pub fn encode_pre_seal(pre_seal: &PreSeal) -> Result<Vec<u8>, MetadataError> {
	match validate_pre_seal(&pre_seal) {
		Ok(_) => Ok([
				&pre_seal.chunk_cid[..],
				&pre_seal.hash[..],
				&pre_seal.sealing_key[..],
				&pre_seal.version[..]
			].concat()),
		Err(err) => Err(err)
	}
}

pub fn decode_pre_seal(pre_seal: &Vec<u8>) -> Result<PreSeal, MetadataError> {
	if pre_seal.len() != PRESEAL_SIZE {
		return Err(MetadataError::PreSealLengthError);
	}

	Ok(PreSeal {
		chunk_cid: pre_seal[0..46].try_into().expect("chunk_cid with incorrect length"),
		hash: pre_seal[46..78].try_into().expect("hash with incorrect length"),
		sealing_key: pre_seal[78..110].try_into().expect("sealing_key with incorrect length"),
		version: pre_seal[110..].try_into().expect("version with incorrect length")
	})
}

pub fn encode_sealed_metadata(sealed_metadata: &SealedMetadata) -> Result<Vec<u8>, MetadataError> {
	match validate_sealed_metadata(&sealed_metadata) {
		Ok(_) => {
			Ok([
				&match sealed_metadata.is_public {
					true => [1u8, 1u8],
					false => [0u8, 0u8],
				}[..],
				&sealed_metadata.cipher[..],
				&sealed_metadata.version[..],
			].concat())
		},
		Err(err) => Err(err)
	}
}

pub fn decode_sealed_metadata(encoded_sealed: &[u8]) -> Result<SealedMetadata, MetadataError> {
	let is_public = match encoded_sealed[0] {
		0 => false,
		1 => true,
		_ => return Err(MetadataError::SealedParseError),
	};

	// get rid of the first type bytes - is_public info
	// get rid of the last 4 bytes - version info
	let cipher = &encoded_sealed[2..encoded_sealed.len() - 4];
	let sealed = SealedMetadata {
		is_public,
		cipher: cipher.to_vec(),
		members_count: match cipher.len() == PRESEAL_SIZE {
			true => 0,
			false => cipher.len() / PRESEAL_ENCRYPTED_SIZE as usize
		} as u64,
		version: encoded_sealed[encoded_sealed.len() - 4..].try_into().map_err(|_| MetadataError::SealedParseError)?,
	};
	Ok(sealed)
}

pub fn decrypt_recovered_cipher(keys: &[BoxSecretKey], cipher: &Vec<u8>) -> Result<Vec<u8>, CryptoError> {
	let mut offset = 0;
	let len = cipher.len();

	for key in keys {
		while offset < len {
			let slice = &cipher[offset..offset + PRESEAL_ENCRYPTED_SIZE];

			// try to decrypt the current chunk of the cipher
			match NaClBox::decrypt(&key, decode_box_cipher(slice)) {

				// we got a hit -> Done
				Ok(res) => return Ok(res),

				// continue, we don't have the key for this one
				//  -> offset the current cipher size
				Err(_) => offset += PRESEAL_ENCRYPTED_SIZE 
			}
		}
	}
	Err(CryptoError::NaClBoxDecryptionFailed)
}

pub fn encode_secretbox_cipher(cipher: &SecretboxCipher) -> Vec<u8> {
	[
		&cipher.0[..],
		&cipher.1[..],
	].concat()
}


pub fn decode_secretbox_cipher(cipher: &[u8]) -> SecretboxCipher {
	// assert!(cipher.len() == PRESEAL_ENCRYPTED_SIZE, MetadataError::SealedParseError);
	(
		cipher[0..24].try_into().expect("unexpected public key len"),
		cipher[24..].try_into().expect("unexpected nonce len"),
	)
}

pub fn encode_box_cipher(box_cipher: &BoxCipher) -> Vec<u8> {
	[
		&box_cipher.0[..],
		&box_cipher.1[..],
		&box_cipher.2[..],
	].concat()
}

pub fn decode_box_cipher(cipher: &[u8]) -> BoxCipher {
	// assert!(cipher.len() == PRESEAL_ENCRYPTED_SIZE, MetadataError::SealedParseError);
	(
		cipher[0..32].try_into().expect("unexpected public key len"),
		cipher[32..56].try_into().expect("unexpected nonce len"),
		cipher[56..].to_vec(),
	)
}

pub fn seal(
	pre_seal: &PreSeal, 
	encryption_schema: &EncryptionSchema, 
) -> Result<SealedMetadata, MetadataError> {

	let mut sealed = SealedMetadata {
		is_public: encryption_schema.get_is_public(),
		cipher: Vec::new(),
		members_count: encryption_schema.get_members_count(),
		version: pre_seal.version,
	};

	let pre_seal_encoded = encode_pre_seal(pre_seal)?;

	for receiver in encryption_schema.get_members() {
		sealed.cipher.append(
			&mut encode_box_cipher(&NaClBox::encrypt(&pre_seal_encoded, * receiver).map_err(|e| MetadataError::CryptoError(e))?)
		);
	}

	Ok(sealed)
}

pub fn unseal(
	sealed: &SealedMetadata, 
	keys: &[BoxSecretKey],
) -> Result<PreSeal, MetadataError> {
	let encoded_pre_seal = decrypt_recovered_cipher(keys, &sealed.cipher).map_err(|e| MetadataError::CryptoError(e))?;
	let pre_seal = decode_pre_seal(&encoded_pre_seal)?;

	Ok(pre_seal)
}

#[derive(Debug, PartialEq, Clone)]
pub struct SecretRecord {
	pub id: Hash,
	pub hash: Hash, 
	pub sealing_key: SecretboxKey,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RecordStore{
	pub store: HashMap<String, SecretRecord>,
}

impl RecordStore {
	pub fn new() -> Self {
		Self{
			store: HashMap::<String, SecretRecord>::new()
		}
	}

	pub fn init(&mut self, path: &PathBuf) {
		let records = FileHandle::read_all(path).unwrap_or(Vec::new());
		let records_len = records.len();
		let mut offset = 0;

		while offset < records_len && offset + 96 <= records_len{
			let cur_records = &records[offset..offset + 96];

			let id = &cur_records[0..32];
			let hash = &cur_records[32..64];
			let sealing_key = &cur_records[64..96];

			self.store.insert(
				crate::utils::encode_hex(id).unwrap(),
				SecretRecord {
					id: id.try_into().unwrap(), 
					hash: hash.try_into().unwrap(), 
					sealing_key: sealing_key.try_into().unwrap(),
				}
			);

			offset += 96;
		}
	}

	pub fn get(&self, id: &Hash) -> Option<&SecretRecord> {
		self.store.get(&crate::utils::encode_hex(id).unwrap())
	}

	pub fn push(&mut self, id: &Hash, hash: &Hash, sealing_key: &SecretboxKey) {
		self.store.insert(
			crate::utils::encode_hex(id).unwrap(),
			SecretRecord {
				id: id.clone(), 
				hash: hash.clone(), 
				sealing_key: sealing_key.clone(),
			}
		);
	}

	pub fn write(&self, path: &PathBuf) {
		let mut records = Vec::new();
		for (_, record) in &self.store {
			let record = [  record.id, record.hash, record.sealing_key].concat();
			records = [&records[..], &record[..]].concat();
		}

		FileHandle::write(&path, &records).expect("writing secret records cannot fail");
	}
}

pub struct EncryptionSchema {
	is_public: bool,
	members: Vec<BoxPublicKey>,
}

impl EncryptionSchema {
	pub fn from_raw(schema: &[u8]) -> Self {
		
		let is_public = match schema[0..2] {
			[0x0, 0x0] => false,
			[0x1, 0x1] => true,
			_ => unreachable!()
		};

		let len = schema.len();
		let mut offset = 2;
		let mut members: Vec<BoxPublicKey> = Vec::new();

		while offset < len && offset +  32 <= len {
			let m: BoxPublicKey = schema[offset..offset + 32].try_into().unwrap();
			members.push(m);
			offset += 32;
		}

		Self {
			is_public,
			members,
		}
	}

	pub fn to_vec(&self) -> Vec<u8> {
		let mut result = Vec::new();
		match self.is_public {
			true => { result.push(0x1); result.push(0x1); },
			false => { result.push(0x0); result.push(0x0); }
		}

		for member in &self.members {
			result = [ &result[..], &member[..] ].concat().to_vec();
		}

		result
	}

	pub fn new(is_public: bool, members: Vec<BoxPublicKey>) -> Self {
		Self {
			is_public,
			members,
		}
	}

	pub fn get_members_count(&self) -> u64 {
		self.members.len() as u64
	}

	pub fn get_members(&self) -> &[BoxPublicKey] {
		&self.members
	}

	pub fn get_is_public(&self) -> bool {
		self.is_public
	}

	pub fn add_member(&mut self, key: BoxPublicKey) {
		self.members.push(key);
	}
}
