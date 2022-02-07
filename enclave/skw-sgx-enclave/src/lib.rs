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
use std::path::PathBuf;
use std::format;

use skw_sgx_protocol::random_bytes;

use sgx_types::{sgx_status_t};
use skw_sgx_protocol::{
    types::{
        metadata::{PROTECTED_FILE_PATH},
        crypto::{BoxSecretKey},
    },
    file::FileHandle,
    metadata::{EncryptionSchema, RecordStore},
};

macro_rules! get_path{
	($id:expr, $path:expr) => ({
		let id_str = skw_sgx_protocol::utils::encode_hex(&$id[..]).unwrap();
        PathBuf::from(format!("./tmp/{}.{}", id_str, $path))
	})
}

#[no_mangle]
pub extern "C" fn unit_test() -> sgx_status_t {
    skw_sgx_protocol::test::skw_unit_test();
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn integration_test_generate_file() -> sgx_status_t {
    // SkyeKiwi Protocol integration Test
    let path: PathBuf = PathBuf::from("./tmp/test_sgx_file");
    let content = random_bytes!(10000);
    
    // TODO: handle this err
    FileHandle::write(&path, &content);

    std::println!("File Generated!");
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn integration_test_compare_file() -> sgx_status_t {
    // SkyeKiwi Protocol integration Test
    let input_path: PathBuf = PathBuf::from("./tmp/test_sgx_file");
    let output_path: PathBuf = PathBuf::from("./tmp/test_sgx_file.down");

    let input = std::sgxfs::read(&input_path).unwrap();
    let output = std::sgxfs::read(&output_path).unwrap();

    if input.len() != output.len() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let len = input.len();
    let matching = input
        .iter()
        .zip(&output)
        .filter(|&(a, b)| a == b)
        .count();

    if matching != len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    std::println!("FILE MATCHES!");
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_upstream_pre(
    id: &mut [u8; 32]
) -> sgx_status_t {
    let new_id = random_bytes!(32);

    let id_path = get_path!(new_id, "up");
    let path: PathBuf = PathBuf::from("./tmp/test_sgx_file");

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(PROTECTED_FILE_PATH));

    // TODO: handle this err
    skw_sgx_protocol::driver::pre_upstream(&path, &new_id, &id_path, &mut records);
    records.write(&PathBuf::from(PROTECTED_FILE_PATH));

    * id = new_id;

    sgx_status_t::SGX_SUCCESS
} 

#[no_mangle]
pub extern "C" fn ecall_protocol_upstream_cid_list(
    cid_ptr: *const u8, cid_len: usize, 
    id: &[u8; 32],
) -> sgx_status_t {
    let cids = unsafe { slice::from_raw_parts(cid_ptr, cid_len) };

    if cids.len() != cid_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    if id.len() != 32 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let output_path: PathBuf = get_path!(id, "up.cid");

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(&PROTECTED_FILE_PATH));
    let sealing_key = match records.get(id) {
        Some(r) => r.sealing_key,
        None => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
    };

    let cid_list_encrypted_raw = skw_sgx_protocol::crypto::NaClSecretBox::encrypt(&sealing_key, &cids).unwrap();
    let cid_list_encrypted = skw_sgx_protocol::metadata::encode_secretbox_cipher(&cid_list_encrypted_raw);

    // TODO: handle this err
    FileHandle::unstrusted_write( &output_path, &cid_list_encrypted, false );
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_upstream_seal(
    cid_ptr: *const u8, cid_len: usize, 
    id: &[u8; 32],
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

    let output_path: PathBuf = get_path!(id, "up.output");

    skw_sgx_protocol::driver::post_upstream(
        &cid_list.try_into().unwrap(), &mut records, id, 
        &EncryptionSchema::from_raw(&encryption_schema_raw),
        &output_path
    ).unwrap();

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_downstream_pre(
    encoded_sealed_ptr: *const u8, encoded_sealed_len: usize, 
    id: &mut [u8; 32], cid: &mut [u8; 46],
) -> sgx_status_t {

    let encoded_sealed = unsafe { slice::from_raw_parts(encoded_sealed_ptr, encoded_sealed_len) };

    if encoded_sealed.len() != encoded_sealed_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    
    let new_id = random_bytes!(32);
    let secret_key: BoxSecretKey = [27, 164, 82, 183, 16, 160, 180, 138, 40, 54, 14, 116, 205, 245, 163, 2, 38, 126, 118, 181, 234, 125, 198, 9, 57, 137, 127, 214, 100, 114, 167, 186];

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(PROTECTED_FILE_PATH));
    let list_cid = skw_sgx_protocol::driver::pre_downstream(
        &encoded_sealed, 
        &[secret_key],
        &new_id, &mut records
    ).unwrap();
    records.write(&PathBuf::from(PROTECTED_FILE_PATH));

    * id = new_id;
    * cid = list_cid;
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_downstream_cid_list(
    encrypted_cid_ptr: *const u8, encrypted_cid_len: usize, 
    id: &[u8; 32],
) -> sgx_status_t {
    let encrypted_cid = unsafe { slice::from_raw_parts(encrypted_cid_ptr, encrypted_cid_len) };

    if encrypted_cid.len() != encrypted_cid_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    if id.len() != 32 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let output_path: PathBuf = get_path!(id, "down.cid");
    
    let mut records = RecordStore::new();
    records.init(&PathBuf::from(&PROTECTED_FILE_PATH));

    let sealing_key = match records.get(id) {
        Some(r) => r.sealing_key,
        None => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
    };

    let cid_list = skw_sgx_protocol::crypto::NaClSecretBox::decrypt(
        &sealing_key, skw_sgx_protocol::metadata::decode_secretbox_cipher(encrypted_cid),
    ).unwrap();

    // TODO: handle this err
    FileHandle::unstrusted_write( &output_path, &cid_list, false );
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_protocol_downstream_unseal(
    id: &[u8; 32]
) -> sgx_status_t {

    let input_path: PathBuf = get_path!(id, "down");
    let output_path: PathBuf = PathBuf::from("./tmp/test_sgx_file.down");

    let mut records = RecordStore::new();
    records.init(&PathBuf::from(PROTECTED_FILE_PATH));
    
    // TODO: handle this err
    skw_sgx_protocol::driver::post_downstream(
        &input_path, &output_path, 
        &id, &mut records
    );
    records.write(&PathBuf::from(PROTECTED_FILE_PATH));

    sgx_status_t::SGX_SUCCESS
}
