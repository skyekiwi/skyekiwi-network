use skw_vm_primitives::contract_runtime::ContractCode;
use skw_vm_primitives::fees::RuntimeFeesConfig;
use skw_vm_primitives::errors::{FunctionCallError, HostError, VMError};

use skw_vm_host::mocks::mock_external::MockedExternal;
use skw_vm_host::types::ReturnData;
use skw_vm_host::{RuntimeExternal, VMConfig};

use crate::runner::WasmiVM;
use crate::tests::{create_context};

#[test]
pub fn test_ts_contract() {
    let code = ContractCode::new(near_test_contracts::ts_contract());
    let mut fake_external = MockedExternal::new();

    let context = create_context(Vec::new());
    let config = VMConfig::test();
    let fees = RuntimeFeesConfig::test();

    // Call method that panics.
    let promise_results = vec![];
    let result = WasmiVM::run(
        &code,
        "try_panic",
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    );
    assert_eq!(
        result.1,
        Some(VMError::FunctionCallError(FunctionCallError::HostError(HostError::GuestPanic {
            panic_msg: "explicit guest panic".to_string()
        })))
    );

    // Call method that writes something into storage.
    let context = create_context(b"foo bar".to_vec());
        WasmiVM::run(
            &code,
            "try_storage_write",
            &mut fake_external,
            context,
            &config,
            &fees,
            &promise_results,
        )
        .0
        .unwrap();
    // Verify by looking directly into the storage of the host.
    {
        let res = fake_external.storage_get(b"foo");
        let value_ptr = res.unwrap().unwrap();
        let value = value_ptr.deref().unwrap();
        let value = String::from_utf8(value).unwrap();
        assert_eq!(value.as_str(), "bar");
    }

    // Call method that reads the value from storage using registers.
    let context = create_context(b"foo".to_vec());
    let result = WasmiVM::run(
        &code,
        "try_storage_read",
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
    );

    if let ReturnData::Value(value) = result.0.unwrap().return_data {
        let value = String::from_utf8(value).unwrap();
        assert_eq!(value, "bar");
    } else {
        panic!("Value was not returned");
    }
}
