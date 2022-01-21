// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::crypto::*;
use skw_crypto::nacl::*;

pub fn box_encrypt_decrypt() {
	let receiver = random_bytes!(32);
	let keypair2 = NaClBox::keypair_from_seed(receiver);
	
	let msg = random_bytes!(100);

	let cipher = NaClBox::encrypt(
		&msg, keypair2.public_key
	).unwrap();

	let decrypted = NaClBox::decrypt(
		&keypair2, cipher
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
