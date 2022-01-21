// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::types::crypto::*;

use x25519_dalek::{StaticSecret, PublicKey};
use xsalsa20poly1305::{
	aead::{Aead, NewAead},
	XSalsa20Poly1305,
};
use std::vec::Vec;

pub struct NaClBox();
pub struct NaClSecretBox();

#[macro_export]
macro_rules! random_bytes{
	($len:expr) => ({
		let mut bytes = [0_u8; $len];
		for byte in bytes.iter_mut() {
			*byte = sgx_rand::random::<u8>();
		}
		bytes
	})
}

impl NaClBox {

	pub fn keypair_from_seed(secret_key: BoxSecretKey) -> BoxKeyPair {
		let public_key:BoxPublicKey = PublicKey::from(&StaticSecret::from(secret_key)).to_bytes();
		BoxKeyPair {
			public_key: public_key,
			secret_key: secret_key
		}
	}

	pub fn encrypt(
		key: &BoxKeyPair,
		msg: &[u8],
		receiver: BoxPublicKey
	) -> Result<BoxCipher, CryptoError> {
		let ecdh = StaticSecret::from(key.secret_key);
		let shared_secret = ecdh.diffie_hellman(
			&PublicKey::from(receiver)
		);
		if let Ok((nonce, cipher_text)) = NaClSecretBox::encrypt(shared_secret.as_bytes(), msg) {
			Ok((key.public_key, nonce, cipher_text))
		} else {
			Err(CryptoError::NaClBoxEncryptionFailed)
		}
	}

	pub fn decrypt(
		key: &BoxKeyPair,
		cipher: BoxCipher,
	) -> Result<Vec<u8>, CryptoError> {
		let (sender, nonce, cipher_text) = cipher;
		let ecdh = StaticSecret::from(key.secret_key);
		let shared_secret = ecdh.diffie_hellman(
			&PublicKey::from(sender)
		);
		NaClSecretBox::decrypt(shared_secret.as_bytes(), (nonce, cipher_text))
	}
}

impl NaClSecretBox {

	pub fn encrypt (
		key: &SecretboxKey,
		msg: &[u8]
	) -> Result<SecretboxCipher, CryptoError> {
		let nonce = sgx_rand::random::<[u8; SECRETBOX_NONCE_LEN]>();

		let secretbox = XSalsa20Poly1305::new(key.into());
		if let Ok(cipher_text) = secretbox.encrypt(&nonce.into(), msg) {
			Ok((nonce, cipher_text))
		} else {
			Err(CryptoError::NaClSecretboxEncryptionFailed)
		}
	}

	pub fn decrypt(
		key: &SecretboxKey,
		cipher: SecretboxCipher
	) -> Result<Vec<u8>, CryptoError> {
		let (nonce, cipher_text) = cipher;

		let secretbox = XSalsa20Poly1305::new(key.into());
		if let Ok(message) = secretbox.decrypt(&nonce.into(), cipher_text.as_ref()) {
			Ok(message)
		} else {
			Err(CryptoError::NaClSecretboxDecryptionFailed)
		}
	}
}
