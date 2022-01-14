use skw_vm_primitives::errors::{CompilationError, FunctionCallError, PrepareError, VMError};
use assert_matches::assert_matches;

use crate::tests::{make_simple_contract_call_vm};

fn initializer_wrong_signature_contract() -> Vec<u8> {
    wat::parse_str(
        r#"
            (module
              (type (;0;) (func (param i32)))
              (func (;0;) (type 0))
              (start 0)
              (export "hello" (func 0))
            )"#,
    )
    .unwrap()
}

#[test]
fn test_initializer_wrong_signature_contract() {
    assert_eq!(
        make_simple_contract_call_vm(&initializer_wrong_signature_contract(), "hello"),
        (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::Deserialization)
            )))
        )
    );
}

fn function_not_defined_contract() -> Vec<u8> {
    wat::parse_str(
        r#"
            (module
              (export "hello" (func 0))
            )"#,
    )
    .unwrap()
}

#[test]
/// StackHeightInstrumentation is weird but it's what we return for now
fn test_function_not_defined_contract() {
    assert_eq!(
        make_simple_contract_call_vm(&function_not_defined_contract(), "hello"),
        (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::Deserialization)
            )))
        )
    );
}

fn function_type_not_defined_contract(bad_type: u64) -> Vec<u8> {
    wat::parse_str(&format!(
        r#"
            (module
              (func (;0;) (type {}))
              (export "hello" (func 0))
            )"#,
        bad_type
    ))
    .unwrap()
}

#[test]
fn test_function_type_not_defined_contract_1() {
    assert_eq!(
        make_simple_contract_call_vm(&function_type_not_defined_contract(1), "hello"),
        (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::Deserialization)
            )))
        )
    );
}

#[test]
// Weird case. It's not valid wasm (wat2wasm validate will fail), but wasmer allows it.
fn test_function_type_not_defined_contract_2() {
    assert_eq!(
        make_simple_contract_call_vm(&function_type_not_defined_contract(0), "hello"),
        (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::Deserialization)
            )))
        )
    );
}

#[test]
fn test_garbage_contract() {
    assert_eq!(
        make_simple_contract_call_vm(&[], "hello"),
        (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::Deserialization)
            )))
        )
    );
}

fn evil_function_index() -> Vec<u8> {
    wat::parse_str(
        r#"
          (module
            (type (;0;) (func))
            (func (;0;) (type 0)
              call 4294967295)
            (export "abort_with_zero" (func 0))
          )"#,
    )
    .unwrap()
}

#[test]
fn test_evil_function_index() {
    assert_eq!(
        make_simple_contract_call_vm(&evil_function_index(), "abort_with_zero"),
        (
            None,
            Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
                CompilationError::PrepareError(PrepareError::Deserialization)
            )))
        )
    );
}

#[test]
fn test_limit_contract_functions_number() {

    let functions_number_limit: u32 = 10_000;
    let method_name = "main";

    let code = near_test_contracts::many_functions_contract(functions_number_limit);
    let (_, err) = make_simple_contract_call_vm(&code, method_name);
    assert_eq!(err, None);

    let code = near_test_contracts::many_functions_contract(functions_number_limit + 1);
    let (_, err) = make_simple_contract_call_vm(&code, method_name);
    
    assert_matches!(
        err,
        Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
            CompilationError::PrepareError(PrepareError::TooManyFunctions)
        )))
    );
}
