// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::slice;
use std::fs::File;
use std::convert::TryInto;
use std::path::PathBuf;
use std::fmt::Write;
use std::string::String;
use std::io::Read;
use std::num::ParseIntError;

mod encryption_schema;
static ENCLAVE_FILE: &'static str = "enclave.signed.so";

fn padded_slice_to_usize(x: &[u8]) -> usize {
	let mut result: usize = 0;
	let mut m = 1;
	for i in x.iter().rev() {
		result += (*i as usize) * m;
		m *= 0x100;
	}
	result
}

pub fn encode_hex(bytes: &[u8]) -> String {
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		write!(&mut s, "{:02x}", b).unwrap();
	}
	s
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}

extern {
    fn unit_test(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
    fn ecall_protocol_upstream_pre(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        hash: &mut [u8; 32]
    ) -> sgx_status_t;
    fn ecall_protocol_upstream_cid_list(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        cid_ptr: *const u8, cid_len: usize,
        id: &[u8; 32],
        output_path_ptr: *const u8, output_path_len: usize, 
    ) -> sgx_status_t;
    fn ecall_protocol_upstream_seal(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        cid_ptr: *const u8, cid_len: usize,
        id: &[u8; 32],
        encryption_schema_ptr: *const u8, encryption_schema_len: usize,
    ) -> sgx_status_t;
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
        // integration_test:: UPSTREAM!
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let mut id = [0u8; 32];
        let init_result = unsafe {
            ecall_protocol_upstream_pre(enclave.geteid(), &mut retval, &mut id)
        };

        let mut all_chunks = Vec::new();

        let chunk_path = PathBuf::from(format!("./tmp/{}", encode_hex(&id)));
        let mut chunks = File::open(chunk_path).expect("chunk file must exist");
        let len = chunks.read_to_end(&mut all_chunks).unwrap_or(0);

        let mut cid_list = Vec::new();
        let mut offset = 0;

        while offset < len && offset + 4 <= len {
            let size_buf = &all_chunks[offset..offset + 4];
            let size = padded_slice_to_usize(size_buf);

            if offset + 4 + size > len {
                // something is wrong. We should never get in here
                break;
            }

            let mut chunk = &all_chunks[offset + 4..offset + 4 + size];
            let encoded_chunk = encode_hex(chunk);
            let result = skw_sgx_ipfs::IpfsClient::add(encoded_chunk.as_bytes().to_vec()).unwrap();
            cid_list = [ &result.cid[..], &cid_list[..] ].concat();

            offset += 8 + size;
        }

        let cid_output_path = std::format!("./tmp/{}.cid", encode_hex(&id));
        let cid_list_result = unsafe {
            ecall_protocol_upstream_cid_list(enclave.geteid(), &mut retval, 
                cid_list.as_ptr() as * const u8, cid_list.len(),
                &id,
                cid_output_path.as_ptr() as * const u8, cid_output_path.len(),
            )
        };

        let mut encrypted_cid = Vec::new();
        let mut cid_output = File::open(cid_output_path).expect("chunk file must exist");
        let len = cid_output.read_to_end(&mut encrypted_cid).unwrap_or(0);

        let mut encryption_schema = encryption_schema::EncryptionSchema::new(false, Vec::new());
        encryption_schema.add_member(decode_hex("1ba452b710a0b48a28360e74cdf5a302267e76b5ea7dc60939897fd66472a7ba").unwrap()[..].try_into().unwrap());
        
        let encryption_schema_str = encryption_schema.to_vec();

        let cid_hex = encode_hex(&encrypted_cid);
        let cid_list_upload_result = skw_sgx_ipfs::IpfsClient::add(cid_hex.as_bytes().to_vec()).unwrap();

        let seal_result = unsafe {
            ecall_protocol_upstream_seal(enclave.geteid(), &mut retval, 
                cid_list_upload_result.cid.as_ptr() as * const u8, cid_list_upload_result.cid.len(),
                &id,
                encryption_schema_str.as_ptr() as * const u8, encryption_schema_str.len(),
            )
        };

        let mut output_sealed = Vec::new();
        let sealed_output_path = PathBuf::from(format!("./tmp/{}.output", encode_hex(&id)));
        let mut sealed_output = File::open(sealed_output_path).expect("chunk file must exist");
        let len = sealed_output.read_to_end(&mut output_sealed).unwrap_or(0);
        let output_sealed_hex = encode_hex(&output_sealed);
        let cid_list_upload_result = skw_sgx_ipfs::IpfsClient::add(output_sealed_hex.as_bytes().to_vec()).unwrap();

        println!("FINAL RESULT CID: {:?}", cid_list_upload_result.cid);

        match seal_result {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                println!("[-] ECALL Enclave Failed {}!", seal_result.as_str());
                return;
            }
        }
    }

    println!("[+] test success...");
    enclave.destroy();
}
