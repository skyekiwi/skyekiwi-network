// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

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
		pub chunk_cid: crate::types::ipfs::CID,
		pub hash: crate::types::file::Hash,
		pub sealing_key: crate::types::crypto::SecretboxKey,
		pub version: [u8; 4]
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct SealedMetadata {
		pub is_public: bool,
		pub cipher: Vec<u8>,
		pub members_count: u64,
		pub version: [u8; 4],
	}

	#[derive(Debug)]
	pub enum MetadataError {
		PreSealLengthError,
		SealedParseError,
		CryptoError(super::crypto::CryptoError),
	}

	pub const PRESEAL_SIZE: usize = 114;
	pub const PRESEAL_ENCRYPTED_SIZE: usize = 186;
	pub const PROTECTED_FILE_PATH: &str = "./skyekiwi_protoccol_cache";
}

pub mod ipfs {
	pub type CID = [u8; 46];
	
	#[derive(Debug, Clone)]
	pub struct IpfsResult {
		cid: CID, size: u64,
	}

	#[derive(Debug)]
	pub enum IpfsError {
		IpfsAddFailed,
		IpfsPinFailed,
		IpfsCatFailed
	}
}

pub mod file {
	use std::vec::Vec;

	pub type Hash = [u8; 32];
	pub type ReadOutput = (Vec<u8>, usize);

	pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

	#[derive(Debug)]
	pub enum FileError {
		FileNotFound,
		HashError,
	}
}

pub mod driver {
	use std::vec::Vec;

	pub type Chunk = Vec<u8>;

	#[derive(Debug)]
	pub enum ProtocolError {
		RecordError,
		MetadataError(super::metadata::MetadataError),
		IpfsError(super::ipfs::IpfsError),
		FileError(super::file::FileError),
		CryptoError(super::crypto::CryptoError),
	}
}