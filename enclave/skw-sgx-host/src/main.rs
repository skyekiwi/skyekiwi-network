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
use std::string::String;
use std::io::Read;
use std::num::ParseIntError;
use std::io::Write;

mod encryption_schema;
static ENCLAVE_FILE: &'static str = "enclave.signed.so";

fn pad_usize(size: usize) -> Vec<u8> {
	let x = format!("{:0>8x}", size);
	decode_hex(&x).unwrap()
}

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
    use std::fmt::Write;
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
    fn integration_test_generate_file(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
    fn integration_test_compare_file(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;

    fn ecall_protocol_upstream_pre(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        id: &mut [u8; 32],
    ) -> sgx_status_t;
    fn ecall_protocol_upstream_cid_list(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        cid_ptr: *const u8, cid_len: usize,
        id: &[u8; 32],
    ) -> sgx_status_t;
    fn ecall_protocol_upstream_seal(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        cid_ptr: *const u8, cid_len: usize,
        id: &[u8; 32],
        encryption_schema_ptr: *const u8, encryption_schema_len: usize,
    ) -> sgx_status_t;

    fn ecall_protocol_downstream_pre(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        encoded_sealed_ptr: *const u8, encoded_sealed_len: usize,
        id: &mut [u8; 32],
        cid: &mut [u8; 46],
    ) -> sgx_status_t;
    fn ecall_protocol_downstream_cid_list(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        encrypted_cid_ptr: *const u8, encrypted_cid_len: usize,
        id: &[u8; 32],
    ) -> sgx_status_t;
    fn ecall_protocol_downstream_unseal(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        id: &[u8; 32],
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

    let mut retval = sgx_status_t::SGX_SUCCESS;
    unsafe {
        integration_test_generate_file(enclave.geteid(), &mut retval);
    };

    let cid = {
        // integration_test:: UPSTREAM!
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let mut id = [0u8; 32];
        let init_result = unsafe {
            ecall_protocol_upstream_pre(enclave.geteid(), &mut retval, &mut id)
        };

        let mut all_chunks = Vec::new();
        let chunk_path = PathBuf::from(format!("./tmp/{}.up", encode_hex(&id)));
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

            offset += 4 + size;
        }

        let cid_list_result = unsafe {
            ecall_protocol_upstream_cid_list(enclave.geteid(), &mut retval, 
                cid_list.as_ptr() as * const u8, cid_list.len(),
                &id,
            )
        };

        let mut encrypted_cid = Vec::new();
        let cid_output_path = PathBuf::from(format!("./tmp/{}.up.cid", encode_hex(&id)));
        let mut cid_output = File::open(cid_output_path).expect("chunk file must exist");
        let len = cid_output.read_to_end(&mut encrypted_cid).unwrap_or(0);


        // A Testing Keypair
        // public_key: [43, 168, 6, 207, 61, 48, 125, 185, 66, 168, 166, 22, 200, 87, 58, 96, 94, 76, 20, 112, 219, 173, 248, 19, 140, 163, 179, 45, 206, 53, 93, 70], 
        // secret_key: [27, 164, 82, 183, 16, 160, 180, 138, 40, 54, 14, 116, 205, 245, 163, 2, 38, 126, 118, 181, 234, 125, 198, 9, 57, 137, 127, 214, 100, 114, 167, 186]
        let mut encryption_schema = encryption_schema::EncryptionSchema::new(false, Vec::new());
        encryption_schema.add_member(
            [43, 168, 6, 207, 61, 48, 125, 185, 66, 168, 166, 22, 200, 87, 58, 96, 94, 76, 20, 112, 219, 173, 248, 19, 140, 163, 179, 45, 206, 53, 93, 70]
        );
        
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
        let sealed_output_path = PathBuf::from(format!("./tmp/{}.up.output", encode_hex(&id)));
        let mut sealed_output = File::open(sealed_output_path).expect("chunk file must exist");
        let len = sealed_output.read_to_end(&mut output_sealed).unwrap_or(0);
        let output_sealed_hex = encode_hex(&output_sealed);
        let cid_list_upload_result = skw_sgx_ipfs::IpfsClient::add(output_sealed_hex.as_bytes().to_vec()).unwrap();

        println!("FINAL RESULT CID: {:?}", cid_list_upload_result.cid);

        cid_list_upload_result.cid
    };

    {
        let encoded_sealed_raw = skw_sgx_ipfs::IpfsClient::cat(cid).unwrap();
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let mut id = [0u8; 32];
        let mut chunk_list_cid = [0u8; 46];

        let encoded_sealed = decode_hex( std::str::from_utf8(&encoded_sealed_raw).unwrap() ).unwrap(); 
        println!("encoded {:?}", encoded_sealed);
        let init_result = unsafe {
            ecall_protocol_downstream_pre(
                enclave.geteid(), &mut retval, 
                encoded_sealed.as_ptr() as * const u8, encoded_sealed.len(),
                &mut id, &mut chunk_list_cid,
            )
        };

        println!("Downstream preporation done!");

        let encrypted_cid_list_raw = skw_sgx_ipfs::IpfsClient::cat(chunk_list_cid.to_vec()).unwrap();
        let encrypted_cid_list = decode_hex( std::str::from_utf8(&encrypted_cid_list_raw).unwrap() ).unwrap(); 

        let init_result = unsafe {
            ecall_protocol_downstream_cid_list(
                enclave.geteid(), &mut retval, 
                encrypted_cid_list.as_ptr() as * const u8, encrypted_cid_list.len(),
                &mut id,
            )
        };

        println!("CID List Extracted!");

        let mut all_chunk_cids = Vec::new();
        let all_chunk_cids_path = PathBuf::from(format!("./tmp/{}.down.cid", encode_hex(&id)));
        let mut all_chunk_cids_file = File::open(all_chunk_cids_path).expect("all_chunk_cids file must exist");
        let len = all_chunk_cids_file.read_to_end(&mut all_chunk_cids).unwrap_or(0);

        let mut all_chunks = Vec::new();

        let mut offset = 0;
        while offset < len && offset + 46 <= len {
            let cur_cid = &all_chunk_cids[offset..offset + 46];
            let cur_chunk_raw = skw_sgx_ipfs::IpfsClient::cat(cur_cid.to_vec()).unwrap();
            let cur_chunk = decode_hex( std::str::from_utf8(&cur_chunk_raw).unwrap() ).unwrap(); 
            let sized = pad_usize(cur_chunk.len());
            all_chunks = [ &all_chunks[..], &sized[..], &cur_chunk[..] ].concat();
            offset += 46;
        }

        let all_chunks_path = PathBuf::from(format!("./tmp/{}.down", encode_hex(&id)));
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(all_chunks_path).unwrap()
            .write_all(&all_chunks);
        // FileHandle::unstrusted_write(all_chunks_path, all_chunks, false);

        let _ = unsafe {
            ecall_protocol_downstream_unseal(
                enclave.geteid(), &mut retval, &id,
            )
        };
    }

    let mut retval = sgx_status_t::SGX_SUCCESS;
    unsafe {
        integration_test_compare_file(enclave.geteid(), &mut retval);
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
