// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
use std::{vec::Vec, convert::TryInto};
use crate::types::{
	metadata::*,
	crypto::{BoxSecretKey, BoxCipher, BoxPublicKey, CryptoError, SecretboxCipher},
};
use crate::crypto::NaClBox;

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

pub fn decode_sealed_metadata(encoded_sealed: &Vec<u8>) -> Result<SealedMetadata, MetadataError> {
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

pub struct EncryptionSchema {
	is_public: bool,
	members: Vec<BoxPublicKey>,
}

impl EncryptionSchema {
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
