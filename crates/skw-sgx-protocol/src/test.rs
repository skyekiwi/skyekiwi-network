// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use sgx_tunittest::*;
use crate::random_bytes;

use std::string::String;
use std::vec::Vec;

pub fn skw_unit_test() {
	rsgx_unit_tests!(
		// metadata
		metadata_test::encode_decode_pre_seal,
		metadata_test::encode_decode_sealed_one,
		metadata_test::encode_decode_sealed_multiple,
		
		// crypto
		crypto_test::secret_box_encrypt_decrypt,
		crypto_test::box_encrypt_decrypt,

		// utils
		utils_test::encode_decode_hex,
		utils_test::pad_uszie,

		// file
		file_test::inflate_client_side,
		file_test::inflate_deflat,
		file_test::sha256_checksum,
	);
}

pub mod file_test {

	use super::*;
	use crate::file::FileHandle;
	pub fn inflate_deflat() {
	  
		let source = [
			84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 103, 108,
			111, 98, 97, 108, 32, 114, 117, 115, 116, 32, 83, 116, 
			114, 105, 110, 103, 32, 105, 110, 105, 116, 32, 98, 121, 
			32, 108, 97, 122, 121, 95, 115, 116, 97, 116, 105, 99, 33
		];

		let deflated = FileHandle::deflate_chunk(&source);
		let recovered = FileHandle::inflate_chunk(&deflated);
		assert_eq!(source.to_vec(), recovered)
	}
  
	pub fn inflate_client_side() {
	  
		let deflated = [
			120, 156,  11, 201, 200,  44,  86,   0, 162,  68,
			133, 244, 156, 252, 164, 196,  28, 133, 162, 210,
			226,  18, 133, 224, 146, 162, 204, 188, 116, 133,
			204, 188, 204,  18, 133, 164,  74, 133, 156, 196,
			170, 202, 248, 226, 146, 196, 146, 204, 100,  69,
			0, 183, 150,  17, 227
		];

		let source = [
			84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 103, 108,
			111, 98, 97, 108, 32, 114, 117, 115, 116, 32, 83, 116, 
			114, 105, 110, 103, 32, 105, 110, 105, 116, 32, 98, 121, 
			32, 108, 97, 122, 121, 95, 115, 116, 97, 116, 105, 99, 33
		];

		let recovered = FileHandle::inflate_chunk(&deflated);
		assert_eq!(source.to_vec(), recovered)
	}
  
  
	pub fn sha256_checksum() {
	  
		let source = [
			84, 104, 105, 115, 32, 105, 115, 32, 97, 
			32, 103, 108, 111, 98, 97, 108, 32, 114, 
			117, 115, 116, 32, 83, 116, 114, 105, 110, 
			103, 32, 105, 110, 105, 116, 32, 98, 121, 
			32, 108, 97, 122, 121, 95, 115, 116, 97, 
			116, 105, 99, 33
		];

		let result = [
			169, 234,  74, 197, 148, 176, 200,
			94, 241, 173, 164, 173,   8, 240,
			249,   3, 191, 104,  87, 236, 224,
			244, 126,  10, 115, 194, 105, 146,
			209,  44, 216,  83
		];

		let hash = FileHandle::sha256_checksum(&source);
		assert_eq!(hash.to_vec(), result)
	}
}
  
pub mod utils_test {
	use crate::utils::{encode_hex, decode_hex, padded_slice_to_usize, pad_usize};
	use super::*;

	use std::println;

	const TEST: &str = "010203040a0b";
	const ANSWER: &[u8] = &[1, 2, 3, 4, 10, 11];

	pub fn encode_decode_hex() {
		// static content
		let answer = decode_hex(TEST);
		assert_eq!(answer.unwrap(), ANSWER);

		let result = encode_hex(ANSWER);
		assert_eq!(result.unwrap(), TEST);

		// random bytes
		let bytes = random_bytes!(40);
		let result = encode_hex(&bytes).unwrap();
		let re_decoded = decode_hex(&result[..]).unwrap();
		assert_eq!(re_decoded, bytes);
	}

	pub fn pad_uszie() {
		// a random len
		let len: usize = 1023 * 1023 * 19;
		let padded_slice = pad_usize(len);
		let recovered = padded_slice_to_usize(&padded_slice);

		assert_eq!(padded_slice.len(), 4);
		assert_eq!(recovered, len);
	}
}

pub mod crypto_test {
	
	use super::*;
	use crate::crypto::{NaClBox, NaClSecretBox};
	
	pub fn box_encrypt_decrypt() {
		let receiver = random_bytes!(32);
		let keypair2 = NaClBox::keypair_from_seed(receiver);
		
		let msg = random_bytes!(100);

		let cipher = NaClBox::encrypt(
			&msg, keypair2.public_key
		).unwrap();

		let decrypted = NaClBox::decrypt(
			&keypair2.secret_key, cipher
		).unwrap();

		assert_eq!(&decrypted[..], &msg[..])
	}

	pub	fn secret_box_encrypt_decrypt() {
		let key = random_bytes!(32);
		let msg = random_bytes!(100);

		let cipher = NaClSecretBox::encrypt(
			&key, &msg
		).unwrap();

		let decrypted = NaClSecretBox::decrypt(
			&key, cipher
		).unwrap();

		assert_eq!(&decrypted[..], &msg[..]);
	}
}

pub mod metadata_test {
	
	use super::*;
	use crate::metadata::{
		encode_pre_seal, encode_box_cipher, encode_sealed_metadata,
		decrypt_recovered_cipher, decode_sealed_metadata, };
	use crate::utils::{decode_hex};
	use crate::crypto::{NaClBox, NaClSecretBox};
	use crate::types::metadata::*;

	use std::{convert::TryInto, vec::Vec};

	const AUTHOR_PRIVATE: &str = "1234567890123456789012345678904512345678901234567890123456789045";
	const RECP_PRIVATE: &str = "1234567890123456789012345678904612345678901234567890123456789046";

	const ENCODED_PRE_SEAL: &str = "516d5a4d7051384b375470315577616538535869335a4a714a444553384a4742694d6d4e575632695261747762570000000000000000000000000000000000000000000000000000000000000000123456789012345678901234567890121234567890123456789012345678904500000101";
	const CHUNK_CID: &str = "QmZMpQ8K7Tp1Uwae8SXi3ZJqJDES8JGBiMmNWV2iRatwbW";
	const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
	const SLK: &str = "1234567890123456789012345678901212345678901234567890123456789045";
	const VERSION: &str = "00000101";

	pub fn encode_decode_pre_seal() {  
		let pre_seal = decode_hex(ENCODED_PRE_SEAL).unwrap();
		let chunk_cid = CHUNK_CID.as_bytes();
		let hash = decode_hex(HASH).unwrap();
		let slk = decode_hex(SLK).unwrap();
		let version = decode_hex(VERSION).unwrap();

		let encoded: Vec<u8> = encode_pre_seal(&PreSeal {
			chunk_cid: chunk_cid.try_into().unwrap(),
			hash: hash.try_into().unwrap(),
			sealing_key: slk.try_into().unwrap(),
			version: version.try_into().unwrap()
		}).unwrap();

		assert_eq!(encoded, pre_seal);
	}

	pub fn encode_decode_sealed_one() {
		let pre_seal = decode_hex(ENCODED_PRE_SEAL).unwrap();
		let author_private = decode_hex(AUTHOR_PRIVATE).unwrap();
		let keypair = NaClBox::keypair_from_seed(author_private.try_into().unwrap());
		let version = decode_hex(VERSION).unwrap();

		let cipher = NaClBox::encrypt(&pre_seal, keypair.public_key).unwrap();
		let cipher_encoded = encode_box_cipher(&cipher);

		let original = SealedMetadata {
			is_public: false,
			cipher: cipher_encoded,
			members_count: 1,
			version: version[..].try_into().expect("version code with incorrect length"),
		};

		let encoded: Vec<u8> = encode_sealed_metadata(&original).unwrap();
		let recovered = decode_sealed_metadata(&encoded).unwrap();
		let recoverd_preseal = decrypt_recovered_cipher(&[keypair.secret_key], &recovered.cipher).unwrap();

		assert_eq!(recovered, original);
		assert_eq!(recoverd_preseal, pre_seal);
	}

	pub fn encode_decode_sealed_multiple() {
		let pre_seal = decode_hex(ENCODED_PRE_SEAL).unwrap();
		let author_private = decode_hex(AUTHOR_PRIVATE).unwrap();
		let recp_private = decode_hex(RECP_PRIVATE).unwrap();

		let keypair = NaClBox::keypair_from_seed(author_private.try_into().unwrap());
		let keypair2 = NaClBox::keypair_from_seed(recp_private.try_into().unwrap());

		let version = decode_hex(VERSION).unwrap();

		let cipher = NaClBox::encrypt(&pre_seal, keypair.public_key).unwrap();
		let cipher_encoded = encode_box_cipher(&cipher);

		let cipher2 = NaClBox::encrypt(&pre_seal, keypair2.public_key).unwrap();
		let cipher2_encoded = encode_box_cipher(&cipher2);

		let original = SealedMetadata {
		is_public: false,
		cipher: [&cipher_encoded[..], &cipher2_encoded[..]].concat(),
		members_count: 2,
		version: version[..].try_into().expect("version code with incorrect length"),
		};

		let encoded: Vec<u8> = encode_sealed_metadata(&original).unwrap();
		let recovered = decode_sealed_metadata(&encoded).unwrap();

		let recoverd_preseal = decrypt_recovered_cipher(&[keypair.secret_key], &recovered.cipher).unwrap();
		let recoverd_preseal_2 = decrypt_recovered_cipher(&[keypair2.secret_key], &recovered.cipher).unwrap();

		assert_eq!(recovered, original);
		assert_eq!(recoverd_preseal, pre_seal);
		assert_eq!(recoverd_preseal_2, pre_seal);
	}
}
