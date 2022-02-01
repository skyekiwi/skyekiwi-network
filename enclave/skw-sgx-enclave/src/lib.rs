// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#![crate_name = "skyekiwiprotocolenclave"]
#![crate_type = "staticlib"]

#![feature(rustc_private)]

extern crate sgx_types;
extern crate skw_sgx_protocol;
extern crate sgx_tstd as std;

use std::println;
use std::slice;
use std::path::PathBuf;
use skw_sgx_protocol::random_bytes;

use sgx_types::{sgx_status_t};
#[macro_use] use skw_sgx_protocol::crypto;
use skw_sgx_protocol::{
    types::{
        driver::{Chunks},
        ipfs::{CID},
    },
    file::FileHandle,
};

#[no_mangle]
pub extern "C" fn unit_test() -> sgx_status_t {
    skw_sgx_protocol::test::skw_unit_test();
    
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn integration_test() -> sgx_status_t {

    // cid_ptr: *const u8, cid_len: usize
    // let cid = unsafe { slice::from_raw_parts(cid_ptr, cid_len) };

    // env_logger::init();
    // SkyeKiwi Protocol integration Test
    let path: PathBuf = PathBuf::from("./test_sgx_file");
    let content = random_bytes!(1000);

    let file = FileHandle::new(path);
    file.write(&content);

    let chunks = skw_sgx_protocol::driver::pre_upstream(&file);

    println!("{:?}", chunks);

    sgx_status_t::SGX_SUCCESS
}
