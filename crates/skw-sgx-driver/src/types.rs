// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

/* Types/Consts for SKW-CRYPTO */
pub mod crypto {
	use std::vec::Vec;

	#[derive(Debug)]
	pub enum CryptoError {
		// tweetnacl errors
		NaClBoxInvalidPublicKey,
		NaClBoxInvalidPrivateKey,
		NaClBoxEncryptionFailed,
		NaClBoxDecryptionFailed,

		NaClSecretboxEncryptionFailed,
		NaClSecretboxDecryptionFailed,
		// TSS errors
		InsufficientShares,
		HeaderError,
	}

	pub const BOX_PUBLIC_KEY_LEN: usize = 32;
	pub const BOX_SECRET_KEY_LEN: usize = 32;

	pub const SECRETBOX_KEY_LEN: usize = 32;
	pub const SECRETBOX_NONCE_LEN: usize = 24;

	pub type BoxPublicKey = [u8; BOX_PUBLIC_KEY_LEN];
	pub type BoxSecretKey = [u8; BOX_SECRET_KEY_LEN];
	pub type SecretboxKey = [u8; SECRETBOX_KEY_LEN];

	#[derive(Clone, Copy, Debug)]
	pub struct BoxKeyPair {
		pub public_key: BoxPublicKey,
		pub secret_key: BoxSecretKey
	}

	// Nonce len of Box & SecretBox are the same :)
	pub type Nonce = [u8; SECRETBOX_NONCE_LEN];

	pub type SecretboxCipher = (Nonce, Vec<u8>);
	pub type BoxCipher = (BoxPublicKey, Nonce, Vec<u8>);
}

pub mod metadata {
	use std::vec::Vec;

	#[derive(Debug, PartialEq, Clone)]
	pub struct PreSeal {
		pub chunk_cid: crate::ipfs::CID,
		pub hash: crate::file::Hash,
		pub sealing_key: crate::crypto::SecretboxKey,
		pub version: [u8; 4]
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct SealedMetadata {
		pub sealed: Sealed,
		pub version: [u8; 4]
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct Sealed {
		pub is_public: bool,
		pub cipher: Vec<u8>,
		pub members_count: u64,
	}

	#[derive(Debug)]
	pub enum MetadataError {
		PreSealLengthError,
		SealedParseError,
	}

	pub const PRESEAL_SIZE: usize = 114;
	pub const PRESEAL_ENCRYPTED_SIZE: usize = 186;
}

pub mod ipfs {
	pub type CID = [u8; 46];
	
	#[derive(Debug, Clone)]
	pub struct IpfsResult {
		cid: CID, 
		size: u64
	}

	#[derive(Debug)]
	pub enum IpfsError {
		IpfsAddFailed,
		IpfsPinFailed,
		IpfsCatFailed
	}
}

pub mod file {
	pub type Hash = [u8; 32];

	#[derive(Debug)]
	pub enum FileError {
		FileNotFound,
	}
}
