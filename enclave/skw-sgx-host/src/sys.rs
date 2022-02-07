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

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    pub fn unit_test(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
    pub fn integration_test_generate_file(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
    pub fn integration_test_compare_file(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;

    pub fn ecall_protocol_upstream_pre(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        id: &mut [u8; 32],
    ) -> sgx_status_t;
    pub fn ecall_protocol_upstream_cid_list(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        cid_ptr: *const u8, cid_len: usize,
        id: &[u8; 32],
    ) -> sgx_status_t;
    pub fn ecall_protocol_upstream_seal(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        cid_ptr: *const u8, cid_len: usize,
        id: &[u8; 32],
        encryption_schema_ptr: *const u8, encryption_schema_len: usize,
    ) -> sgx_status_t;

    pub fn ecall_protocol_downstream_pre(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        encoded_sealed_ptr: *const u8, encoded_sealed_len: usize,
        id: &mut [u8; 32],
        cid: &mut [u8; 46],
    ) -> sgx_status_t;
    pub fn ecall_protocol_downstream_cid_list(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        encrypted_cid_ptr: *const u8, encrypted_cid_len: usize,
        id: &[u8; 32],
    ) -> sgx_status_t;
    pub fn ecall_protocol_downstream_unseal(
        eid: sgx_enclave_id_t, 
        retval: *mut sgx_status_t, 
        id: &[u8; 32],
    ) -> sgx_status_t;
}

pub fn init_enclave() -> SgxResult<SgxEnclave> {
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
