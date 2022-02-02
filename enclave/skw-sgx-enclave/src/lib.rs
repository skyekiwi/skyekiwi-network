// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#![crate_name = "skyekiwiprotocolenclave"]
#![crate_type = "staticlib"]

#![feature(rustc_private)]

extern crate sgx_types;
extern crate skw_sgx_protocol;
extern crate sgx_tstd as std;
use core::convert::TryInto;
use std::slice;
use std::vec::Vec;
use std::path::PathBuf;
use std::format;

use skw_sgx_protocol::random_bytes;

use sgx_types::{sgx_status_t};
#[macro_use] use skw_sgx_protocol::crypto;
use skw_sgx_protocol::{
    types::{
        driver::{Chunks},
        ipfs::{CID},
        file::Hash,
        metadata::{PROTECTED_FILE_PATH},
    },
    file::FileHandle,
    metadata::{EncryptionSchema, RecordStore},
    utils::encode_hex,
};

#[no_mangle]
pub extern "C" fn unit_test() -> sgx_status_t {
    skw_sgx_protocol::test::skw_unit_test();
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_upstream_pre(
    hash: &mut [u8; 32]
) -> sgx_status_t {

    // SkyeKiwi Protocol integration Test
    let path: PathBuf = PathBuf::from("./tmp/test_sgx_file");
    let mut content = random_bytes!(10000);
    FileHandle::write(&path, &content);

    let id = random_bytes!(32);
    let id_str = skw_sgx_protocol::utils::encode_hex(&id[..]).unwrap();
    let id_path: PathBuf = PathBuf::from(format!("./tmp/{}", id_str));

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(PROTECTED_FILE_PATH));

    skw_sgx_protocol::driver::pre_upstream(&path, &id, &id_path, &mut records);
    records.write(&PathBuf::from(PROTECTED_FILE_PATH));

    * hash = id;

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_upstream_cid_list(
    cid_ptr: *const u8, cid_len: usize, 
    id: &[u8; 32],
    output_path_ptr: *const u8, output_path_len: usize, 
) -> sgx_status_t {
    let cids = unsafe { slice::from_raw_parts(cid_ptr, cid_len) };

    if cids.len() != cid_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let output_path = unsafe { slice::from_raw_parts(output_path_ptr, output_path_len) };
    if output_path.len() != output_path_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(&PROTECTED_FILE_PATH));
    let sealing_key = match records.get(id) {
        Some(r) => r.sealing_key,
        None => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
    };

    let cid_list_encrypted_raw = skw_sgx_protocol::crypto::NaClSecretBox::encrypt(&sealing_key, &cids).unwrap();
    let cid_list_encrypted = skw_sgx_protocol::metadata::encode_secretbox_cipher(&cid_list_encrypted_raw);

    FileHandle::unstrusted_write(
        &PathBuf::from(std::str::from_utf8(&output_path).unwrap()),
        &cid_list_encrypted,
        false
    );

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_upstream_seal(
    cid_ptr: *const u8, cid_len: usize, id: &[u8; 32],
    encryption_schema_ptr: *const u8, encryption_schema_len: usize,
) -> sgx_status_t {
    let cid_list = unsafe { slice::from_raw_parts(cid_ptr, cid_len) };

    if cid_list.len() != cid_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let encryption_schema_raw = unsafe { slice::from_raw_parts(encryption_schema_ptr, encryption_schema_len) };

    if encryption_schema_raw.len() != encryption_schema_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(PROTECTED_FILE_PATH));

    let (hash, sealing_key) = match records.get(id) {
        Some(r) => (r.hash, r.sealing_key),
        None => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
    };

    let output_path = PathBuf::from(format!("./tmp/{}.output", encode_hex(id).unwrap()));
    skw_sgx_protocol::driver::post_upstream(
        &cid_list.try_into().unwrap(), &mut records, id, 
        &EncryptionSchema::from_raw(&encryption_schema_raw),
        &output_path
    ).unwrap();

    sgx_status_t::SGX_SUCCESS
}
