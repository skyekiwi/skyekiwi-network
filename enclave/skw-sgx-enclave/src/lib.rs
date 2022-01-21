// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#![crate_name = "skyekiwiprotocolenclave"]
#![crate_type = "staticlib"]

#![feature(rustc_private)]

extern crate sgx_types;
use sgx_types::{sgx_status_t};

#[no_mangle]
pub extern "C" fn say_something() -> sgx_status_t {

    env_logger::init();
    skw_sgx_protocol::test::skw_unit_test();

    // let msg = random_bytes!(100);
    // let seed1 = random_bytes!(32);
    // let seed2 = random_bytes!(32);

    // let keypair1 = skw_crypto::nacl::NaClBox::keypair_from_seed(seed1);
    // let keypair2 = skw_crypto::nacl::NaClBox::keypair_from_seed(seed2);

    // let mut cipher = skw_crypto::nacl::NaClBox::encrypt(
    //     &keypair1, &msg, keypair2.public_key
    // ).unwrap();
            
    // // let decrypted = skw_crypto::nacl::NaClBox::decrypt(
    // //     &keypair2, &mut cipher, keypair1.public_key
    // // ).unwrap();

    // info!("{:?}", cipher);
    // // info!("{:?}", decrypted);

    sgx_status_t::SGX_SUCCESS
}
