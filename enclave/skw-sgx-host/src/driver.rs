use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::fs::File;
use std::convert::TryInto;
use std::path::PathBuf;
use std::string::String;
use std::num::ParseIntError;
use std::io::{Read, Write};

use crate::sys::*;
use crate::encryption_schema::EncryptionSchema;
use crate::utils::{encode_hex, decode_hex, padded_slice_to_usize, pad_usize};

pub type CID = [u8; 46];

macro_rules! get_path{
	($id:expr, $path:expr) => ({
		let id_str = crate::utils::encode_hex(&$id[..]);
        PathBuf::from(format!("./tmp/{}.{}", id_str, $path))
	})
}


pub fn upstream(enclave: &SgxEnclave) -> CID {
    // integration_test:: UPSTREAM!

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let mut id = [0u8; 32];
    let init_result = unsafe {
        ecall_protocol_upstream_pre(enclave.geteid(), &mut retval, &mut id)
    };

    let mut all_chunks = Vec::new();
    let chunk_path = get_path!(id, "up");
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
    let cid_output_path = get_path!(id, "up.cid");
    let mut cid_output = File::open(cid_output_path).expect("chunk file must exist");
    let len = cid_output.read_to_end(&mut encrypted_cid).unwrap_or(0);


    // A Testing Keypair
    // public_key: [43, 168, 6, 207, 61, 48, 125, 185, 66, 168, 166, 22, 200, 87, 58, 96, 94, 76, 20, 112, 219, 173, 248, 19, 140, 163, 179, 45, 206, 53, 93, 70], 
    // secret_key: [27, 164, 82, 183, 16, 160, 180, 138, 40, 54, 14, 116, 205, 245, 163, 2, 38, 126, 118, 181, 234, 125, 198, 9, 57, 137, 127, 214, 100, 114, 167, 186]
    let mut encryption_schema = EncryptionSchema::new(false, Vec::new());
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
    let sealed_output_path = get_path!(id, "up.output");
    let mut sealed_output = File::open(sealed_output_path).expect("chunk file must exist");
    let len = sealed_output.read_to_end(&mut output_sealed).unwrap_or(0);
    let output_sealed_hex = encode_hex(&output_sealed);
    let cid_list_upload_result = skw_sgx_ipfs::IpfsClient::add(output_sealed_hex.as_bytes().to_vec()).unwrap();

    // cannot fail
    cid_list_upload_result.cid.try_into().unwrap()
}

pub fn downstream(enclave: &SgxEnclave, cid: CID) {
    let encoded_sealed_raw = skw_sgx_ipfs::IpfsClient::cat(cid.to_vec()).unwrap();
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let mut id = [0u8; 32];
    let mut chunk_list_cid = [0u8; 46];

    let encoded_sealed = decode_hex( std::str::from_utf8(&encoded_sealed_raw).unwrap() ).unwrap(); 
    let init_result = unsafe {
        ecall_protocol_downstream_pre(
            enclave.geteid(), &mut retval, 
            encoded_sealed.as_ptr() as * const u8, encoded_sealed.len(),
            &mut id, &mut chunk_list_cid,
        )
    };

    let encrypted_cid_list_raw = skw_sgx_ipfs::IpfsClient::cat(chunk_list_cid.to_vec()).unwrap();
    let encrypted_cid_list = decode_hex( std::str::from_utf8(&encrypted_cid_list_raw).unwrap() ).unwrap(); 

    let init_result = unsafe {
        ecall_protocol_downstream_cid_list(
            enclave.geteid(), &mut retval, 
            encrypted_cid_list.as_ptr() as * const u8, encrypted_cid_list.len(),
            &mut id,
        )
    };

    let mut all_chunk_cids = Vec::new();
    
    let all_chunk_cids_path = get_path!(id, "down.cid");
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

    let all_chunks_path = get_path!(id, "down");
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