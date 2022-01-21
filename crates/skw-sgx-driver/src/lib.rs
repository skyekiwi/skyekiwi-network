// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
extern crate sgx_tstd as std;
extern crate sgx_rand;

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

pub mod nacl;
