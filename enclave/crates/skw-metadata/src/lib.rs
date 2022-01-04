// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
use sgx_tstd::{vec::Vec, convert::TryInto};
use skw_types::{
	metadata::*,
	crypto::{BoxKeyPair, BoxCipher, CryptoError}
};
use skw_crypto::nacl::NaClBox;

pub fn validate_pre_seal(_pre_seal: &PreSeal) -> Result<(), MetadataError> {
	Ok(())
}
pub fn validate_sealed_metadata(_sealed: &SealedMetadata) -> Result<(), MetadataError> {
	Ok(())
}

pub fn encode_pre_seal(pre_seal: PreSeal) -> Result<Vec<u8>, MetadataError> {
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

pub fn decode_pre_seal(pre_seal: Vec<u8>) -> Result<PreSeal, MetadataError> {
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

pub fn encode_sealed_metadata(sealed_metadata: SealedMetadata) -> Result<Vec<u8>, MetadataError> {
	match validate_sealed_metadata(&sealed_metadata) {
		Ok(_) => {
			Ok([
				&match sealed_metadata.sealed.is_public {
					true => [1u8, 1u8],
					false => [0u8, 0u8],
				}[..],
				&sealed_metadata.sealed.cipher[..],
				&sealed_metadata.version[..],
			].concat())
		},
		Err(err) => Err(err)
	}
}

pub fn decode_sealed_metadata(encoded_sealed: Vec<u8>) -> Result<SealedMetadata, MetadataError> {
	let is_public = match encoded_sealed[0] {
		0 => false,
		1 => true,
		_ => return Err(MetadataError::SealedParseError),
	};

	let cipher = &encoded_sealed[2..encoded_sealed.len() - 4];

	Ok(SealedMetadata {
		sealed: Sealed {
			is_public,
			cipher: cipher.to_vec(),
			members_count: match cipher.len() == PRESEAL_SIZE {
				true => 0,
				false => cipher.len() / PRESEAL_ENCRYPTED_SIZE as usize
			} as u64,
		},
		version: encoded_sealed[encoded_sealed.len() - 4..].try_into().expect("version code with incorrect length"),
	})
}

pub fn decrypt_recovered_cipher(keys: &[BoxKeyPair], cipher: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
	let mut offset = 0;
	let len = cipher.len();

	for key in keys {
		while offset < len {
			let slice = &cipher[offset..offset + PRESEAL_ENCRYPTED_SIZE];

			match NaClBox::decrypt(&key, decode_box_cipher(slice)) {
				Ok(res) => return Ok(res),
				Err(_) => offset += PRESEAL_ENCRYPTED_SIZE 
			}
		}
	}
	Err(CryptoError::NaClBoxDecryptionFailed)
}

pub fn encode_box_cipher(box_cipher: BoxCipher) -> Vec<u8> {
	[
		&box_cipher.0[..],
		&box_cipher.1[..],
		&box_cipher.2[..],
	].concat()
}

pub fn decode_box_cipher(cipher: &[u8]) -> BoxCipher {
	(
		cipher[0..32].try_into().expect("unexpected public key len"),
		cipher[32..56].try_into().expect("unexpected nonce len"),
		cipher[56..].to_vec(),
	)
}
