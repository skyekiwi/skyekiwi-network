// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;

mod encryption_schema;
mod driver;
mod sys;
mod utils;

use crate::utils::*;

fn main() {
    let enclave = match sys::init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };

    {
        // unit_test
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let result = unsafe {
            sys::unit_test(enclave.geteid(), &mut retval)
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                println!("[-] ECALL Enclave Failed {}!", result.as_str());
                return;
            }
        }
    }

    let mut retval = sgx_status_t::SGX_SUCCESS;
    unsafe {
        sys::integration_test_generate_file(enclave.geteid(), &mut retval);
    };

    let cid = crate::driver::upstream(&enclave);

    crate::driver::downstream(&enclave, cid);

    let mut retval = sgx_status_t::SGX_SUCCESS;
    unsafe {
        sys::integration_test_compare_file(enclave.geteid(), &mut retval);
    };
    
    match retval {
        sgx_status_t::SGX_SUCCESS => {},
        _ => {
            println!("[-] File Comparison Failed {}!", retval.as_str());
            return;
        }
    }

    println!("[+] test success...");
    enclave.destroy();
}
