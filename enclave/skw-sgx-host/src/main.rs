// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn unit_test(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
    fn integration_test(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

fn main() {
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };

    // IPFS Client Test
    const CONTENT: &str = "some random string ...";
    let result = skw_sgx_ipfs::IpfsClient::add(CONTENT.as_bytes().to_vec()).unwrap();
    let recovered = skw_sgx_ipfs::IpfsClient::cat(result.cid).unwrap();
    println!("{:?}", String::from_utf8(recovered));

    {
        // unit_test
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let result = unsafe {
            unit_test(enclave.geteid(), &mut retval)
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                println!("[-] ECALL Enclave Failed {}!", result.as_str());
                return;
            }
        }
    }

    {
        // integration_test
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let result = unsafe {
            integration_test(enclave.geteid(), &mut retval)
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                println!("[-] ECALL Enclave Failed {}!", result.as_str());
                return;
            }
        }
    }

    println!("[+] unit_test success...");
    enclave.destroy();
}
