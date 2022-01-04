// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#[cfg(target_env = "sgx")]
extern crate core;

extern crate sgx_rand;
#[macro_use] extern crate sgx_tstd as std;
extern crate sgx_tunittest;

use sgx_tunittest::*;
use std::vec::Vec;
use std::string::String;

mod crypto;
use crypto::*;

mod utils;
use utils::*;

mod metadata;
use metadata::*;

mod file;
use file::*;

pub fn skw_unit_test() {
	rsgx_unit_tests!(
		// metadata
		encode_decode_pre_seal,
		encode_decode_sealed_one,
		encode_decode_sealed_multiple,
		// crypto
		secret_box_encrypt_decrypt,
		box_encrypt_decrypt,

		// utils
		encode_decode_hex,

		// file
		inflate_client_side,
		inflate_deflat,
		sha256_checksum,
	);
}
