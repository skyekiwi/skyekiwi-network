//! Host function interface for smart contracts.
//!
//! Besides native WASM operations, smart contracts can call into runtime to
//! gain access to extra functionality, like operations with store. Such
//! "extras" are called "Host function", and play a role similar to syscalls. In
//! this module, we integrate host functions with various wasm runtimes we
//! support. The actual definitions of host functions live in the `vm-logic`
//! crate.
//!
//! Basically, what the following code does is (in pseudo-code):
//!
//! ```ignore
//! for host_fn in all_host_functions {
//!    wasm_imports.define("env", host_fn.name, |args| host_fn(args))
//! }
//! ```
//!
//! The actual implementation is a bit more complicated, for two reasons. First,
//! host functions have different signatures, so there isn't a trivial single
//! type one can use to hold a host function. Second, we want to use direct
//! calls in the compiled WASM, so we need to avoid dynamic dispatch and hand
//! functions as ZSTs to the WASM runtimes. This basically means that we need to
//! code the above for-loop as a macro.
//!
//! So, the `imports!` macro invocation is the main "public" API -- it just list
//! all host functions with their signatures. `imports! { foo, bar, baz }`
//! expands to roughly
//!
//! ```ignore
//! macro_rules! for_each_available_import {
//!    $($M:ident) => {
//!        $M!(foo);
//!        $M!(bar);
//!        $M!(baz);
//!    }
//! }
//! ```
//!
//! That is, `for_each_available_import` is a high-order macro which takes macro
//! `M` as a parameter, and calls `M!` with each import. Each supported WASM
//! runtime (see submodules of this module) then calls
//! `for_each_available_import` with its own import definition logic.
//!
//! The real `for_each_available_import` takes one more argument --
//! `protocol_version`. We can add new imports, but we must make sure that they
//! are only available to contracts at a specific protocol version -- we can't
//! make imports retroactively available to old transactions. So
//! `for_each_available_import` takes care to invoke `M!` only for currently
//! available imports.

// macro_rules! imports {
//     (
//       $($(#[$stable_feature:ident])? $(#[$feature_name:literal, $feature:ident])*
//         $func:ident < [ $( $arg_name:ident : $arg_type:ident ),* ] -> [ $( $returns:ident ),* ] >,)*
//     ) => {
//         macro_rules! for_each_available_import {
//             ($protocol_version:ident, $M:ident) => {$(
//                 $(#[cfg(feature = $feature_name)])*
//                 if true
//                     $(&& near_primitives::checked_feature!($feature_name, $feature, $protocol_version))*
//                     $(&& near_primitives::checked_feature!("stable", $stable_feature, $protocol_version))?
//                 {
//                     $M!($func < [ $( $arg_name : $arg_type ),* ] -> [ $( $returns ),* ] >);
//                 }
//             )*}
//         }
//     }
// }


macro_rules! index_to_field_name {
  (0) => ("read_register");
  (1) => ("register_len");
  (2) => ("write_register");
  (3) => ("current_account_id");
  (4) => ("signer_account_id");
  (5) => ("signer_account_pk");
  (6) => ("predecessor_account_id");
  (7) => ("input");
  (8) => ("block_index");
  (9) => ("block_timestamp");
  (10) => ("epoch_height");
  (11) => ("storage_usage");
  (12) => ("account_balance");
  (13) => ("account_locked_balance");
  (14) => ("attached_deposit");
  (15) => ("prepaid_gas");
  (16) => ("used_gas");
  (17) => ("random_seed");
  (18) => ("sha256");
  (19) => ("keccak256");
  (20) => ("keccak512");
  (21) => ("ripemd160");
  (22) => ("ecrecover");
  (23) => ("value_return");
  (24) => ("panic");
  (25) => ("panic_utf8");
  (26) => ("log_utf8");
  (27) => ("log_utf16");
  (28) => ("abort");
  (29) => ("promise_create");
  (30) => ("promise_then");
  (31) => ("promise_and");
  (32) => ("promise_batch_create");
  (33) => ("promise_batch_then");
  (34) => ("promise_batch_action_create_account");
  (35) => ("promise_batch_action_deploy_contract");
  (36) => ("promise_batch_action_function_call");
  (37) => ("promise_batch_action_transfer");
  (38) => ("promise_batch_action_stake");
  (39) => ("promise_batch_action_add_key_with_full_access");
  (40) => ("promise_batch_action_add_key_with_function_call");
  (41) => ("promise_batch_action_delete_key");
  (42) => ("promise_batch_action_delete_account");
  (43) => ("promise_results_count");
  (44) => ("promise_result");
  (45) => ("promise_return");
  (46) => ("storage_write");
  (47) => ("storage_read");
  (48) => ("storage_remove");
  (49) => ("storage_has_key");
  (50) => ("storage_iter_prefix");
  (51) => ("storage_iter_range");
  (52) => ("storage_iter_next");
  (53) => ("gas");
  (54) => ("validator_stake");
  (55) => ("validator_total_stake");
}

macro_rules! filed_name_to_index {
  ("read_register") => {0};
  ("register_len") => {1};
  ("write_register") => {2};
  ("current_account_id") => {3};
  ("signer_account_id") => {4};
  ("signer_account_pk") => {5};
  ("predecessor_account_id") => {6};
  ("input") => {7};
  ("block_index") => {8};
  ("block_timestamp") => {9};
  ("epoch_height") => {10};
  ("storage_usage") => {11};
  ("account_balance") => {12};
  ("account_locked_balance") => {13};
  ("attached_deposit") => {14};
  ("prepaid_gas") => {15};
  ("used_gas") => {16};
  ("random_seed") => {17};
  ("sha256") => {18};
  ("keccak256") => {19};
  ("keccak512") => {20};
  ("ripemd160") => {21};
  ("ecrecover") => {22};
  ("value_return") => {23};
  ("panic") => {24};
  ("panic_utf8") => {25};
  ("log_utf8") => {26};
  ("log_utf16") => {27};
  ("abort") => {28};
  ("promise_create") => {29};
  ("promise_then") => {30};
  ("promise_and") => {31};
  ("promise_batch_create") => {32};
  ("promise_batch_then") => {33};
  ("promise_batch_action_create_account") => {34};
  ("promise_batch_action_deploy_contract") => {35};
  ("promise_batch_action_function_call") => {36};
  ("promise_batch_action_transfer") => {37};
  ("promise_batch_action_stake") => {38};
  ("promise_batch_action_add_key_with_full_access") => {39};
  ("promise_batch_action_add_key_with_function_call") => {40};
  ("promise_batch_action_delete_key") => {41};
  ("promise_batch_action_delete_account") => {42};
  ("promise_results_count") => {43};
  ("promise_result") => {44};
  ("promise_return") => {45};
  ("storage_write") => {46};
  ("storage_read") => {47};
  ("storage_remove") => {48};
  ("storage_has_key") => {49};
  ("storage_iter_prefix") => {50};
  ("storage_iter_range") => {51};
  ("storage_iter_next") => {52};
  ("gas") => {53};
  ("validator_stake") => {54};
  ("validator_total_stake") => {55};
}

macro_rules! imports {
    (
      $($func:ident < [ $( $arg_name:ident : $arg_type:ident ),* ] -> [ $( $returns:ident ),* ] >,)*
    ) => {
        macro_rules! for_each_available_import {
            ($M:ident) => {$(
                $M!($func < [ $( $arg_name : $arg_type ),* ] -> [ $( $returns ),* ] >);
            )*}
        }
    }
}

macro_rules! wasm_to_rust_types {
    (I32) => {u32};
    (I64) => {u64};
    () => ();
}

imports! {
    // #############
    // # Registers #
    // #############
    read_register<[0: I64, 1: I64] -> []>,
    register_len<[0: I64] -> [I64]>,
    write_register<[0: I64, 1: I64, 2: I64] -> []>,
    // ###############
    // # Context API #
    // ###############
    current_account_id<[0: I64] -> []>,
    signer_account_id<[0: I64] -> []>,
    signer_account_pk<[0: I64] -> []>,
    predecessor_account_id<[0: I64] -> []>,
    input<[0: I64] -> []>,
    block_index<[] -> [I64]>,
    block_timestamp<[] -> [I64]>,
    epoch_height<[] -> [I64]>,
    storage_usage<[] -> [I64]>,
    // #################
    // # Economics API #
    // #################
    account_balance<[0: I64] -> []>,
    account_locked_balance<[0: I64] -> []>,
    attached_deposit<[0: I64] -> []>,
    prepaid_gas<[] -> [I64]>,
    used_gas<[] -> [I64]>,
    // ############
    // # Math API #
    // ############
    random_seed<[0: I64] -> []>,
    sha256<[0: I64, 1: I64, 2: I64] -> []>,
    keccak256<[0: I64, 1: I64, 2: I64] -> []>,
    keccak512<[0: I64, 1: I64, 2: I64] -> []>,
    #[MathExtension] ripemd160<[0: I64, 1: I64, 2: I64] -> []>,
    #[MathExtension] ecrecover<[0: I64, 1: I64, 2: I64, 3: I64, 4: I64, 5: I64, 6: I64] -> [I64]>,
    // #####################
    // # Miscellaneous API #
    // #####################
    value_return<[0: I64, 1: I64] -> []>,
    panic<[] -> []>,
    panic_utf8<[0: I64, 1: I64] -> []>,
    log_utf8<[0: I64, 1: I64] -> []>,
    log_utf16<[0: I64, 1: I64] -> []>,
    abort<[0: I32, 1: I32, 2: I32, 3: I32] -> []>,
    // ################
    // # Promises API #
    // ################
    promise_create<[
        0: I64,
        1: I64,
        2: I64,
        3: I64,
        4: I64,
        5: I64,
        6: I64,
        7: I64
    ] -> [I64]>,
    promise_then<[
        0: I64,
        1: I64,
        2: I64,
        3: I64,
        4: I64,
        5: I64,
        6: I64,
        7: I64,
        8: I64
    ] -> [I64]>,
    promise_and<[0: I64, 1: I64] -> [I64]>,
    promise_batch_create<[0: I64, 1: I64] -> [I64]>,
    promise_batch_then<[0: I64, 1: I64, 2: I64] -> [I64]>,
    // #######################
    // # Promise API actions #
    // #######################
    promise_batch_action_create_account<[0: I64] -> []>,
    promise_batch_action_deploy_contract<[0: I64, 1: I64, 2: I64] -> []>,
    promise_batch_action_function_call<[
        0: I64,
        1: I64,
        2: I64,
        3: I64,
        4: I64,
        5: I64,
        6: I64
    ] -> []>,
    promise_batch_action_transfer<[0: I64, 1: I64] -> []>,
    promise_batch_action_stake<[
        0: I64,
        1: I64,
        2: I64,
        3: I64
    ] -> []>,
    promise_batch_action_add_key_with_full_access<[
        0: I64,
        1: I64,
        2: I64,
        3: I64
    ] -> []>,
    promise_batch_action_add_key_with_function_call<[
        0: I64,
        1: I64,
        2: I64,
        3: I64,
        4: I64,
        5: I64,
        6: I64,
        7: I64,
        8: I64
    ] -> []>,
    promise_batch_action_delete_key<[
        0: I64,
        1: I64,
        2: I64
    ] -> []>,
    promise_batch_action_delete_account<[
        0: I64,
        1: I64,
        2: I64
    ] -> []>,
    // #######################
    // # Promise API results #
    // #######################
    promise_results_count<[] -> [I64]>,
    promise_result<[0: I64, 1: I64] -> [I64]>,
    promise_return<[0: I64] -> []>,
    // ###############
    // # Storage API #
    // ###############
    storage_write<[0: I64, 1: I64, 2: I64, value_ptr: I64, 3: I64] -> [I64]>,
    storage_read<[0: I64, 1: I64, 2: I64] -> [I64]>,
    storage_remove<[0: I64, 1: I64, 2: I64] -> [I64]>,
    storage_has_key<[0: I64, 1: I64] -> [I64]>,
    storage_iter_prefix<[0: I64, 1: I64] -> [I64]>,
    storage_iter_range<[0: I64, 1: I64, 2: I64, 3: I64] -> [I64]>,
    storage_iter_next<[0: I64, 1: I64, 2: I64] -> [I64]>,
    // Function for the injected gas counter. Automatically called by the gas meter.
    gas<[0: I32] -> []>,
    // ###############
    // # Validator API #
    // ###############
    validator_stake<[0: I64, 1: I64, 2: I64] -> []>,
    validator_total_stake<[0: I64] -> []>,
}

pub(crate) mod wasmi_import {
    use near_vm_logic::{ProtocolVersion, VMLogic, VMLogicError};
    use wasmi::{
        Externals, RuntimeValue, RuntimeArgs, Error, ModuleImportResolver,
        FuncRef, Signature, FuncInstance, Trap, ValueType, ImportsBuilder,
    };

    #[derive(Clone)]
    struct HostExternals {}
    
    impl Externals for HostExternals {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            macro_rules! add_impl_by_index {
                (
                $func:ident < [ $( $arg_name:ident : $arg_type:ident ),* ] -> [ $( $returns:ident ),* ] >
                ) => {
                    filed_name_to_index!(stringify!($func)) => {
                        const IS_GAS: bool = str_eq(stringify!($func), "gas");
                        let _span = if IS_GAS {
                            None
                        } else {
                            Some(tracing::trace_span!(target: "host-function", stringify!($func)).entered())
                        };
                        let logic: &mut VMLogic = unsafe { &mut *(args.nth_checked(0)?.logic as *mut VMLogic<'_>) };
                        let out = logic.$func( $( args.nth_checked($arg_name)?, )* );
                        Ok(Some(RuntimeValue::$returns(result as wasm_to_rust_types!($returns))))
                    }
                };
            }

            match index {
                for_each_available_import!(add_impl_by_index)
                // _ => panic!("Unimplemented function at {}", index),
            }
        }
    }
    
    impl ModuleImportResolver for HostExternals {
        fn resolve_func(
            &self,
            field_name: &str,
            signature: &Signature
        ) -> Result<FuncRef, Error> {
            macro_rules! get_index_from_name {
                (
                $func:ident < [ $( $arg_name:ident : $arg_type:ident ),* ] -> [ $( $returns:ident ),* ] >
                ) => {
                    stringify!($func) => index_to_field_name!(stringify!($func))
                };
            }

            let index = match field_name {
                for_each_available_import!(get_index_from_name)
                // _ => {
                //     return Err(Error::Instantiation(
                //         format!("Export {} not found", field_name),
                //     ))
                // }
            };

            macro_rules! get_sig_by_index {
                (
                $func:ident < [ $( $arg_name:ident : $arg_type:ident ),* ] -> [ $( $returns:ident ),* ] >
                ) => {
                    filed_name_to_index!(stringify!($func)) => (&[$( ValueType($arg_type)?, )*], Some(ValueType::$returns))
                };
            }
            let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
                for_each_available_import!(get_sig_by_index)               
                // _ => return false,
            };

            if !(signature.params() == params && signature.return_type() == ret_ty) {
                return Err(Error::Instantiation(
                    format!("Export {} has a bad signature", field_name)
                ));
            }

            Ok(FuncInstance::alloc_host(
                Signature::new(params, ret_ty),
                index,
            ))
        }
    }


    pub(crate) fn build<'a>(
        logic: &'a mut VMLogic<'_>,
    ) ->  ImportsBuilder<'a> {
        ImportsBuilder::new()
            .with_resolver("env", &HostExternals)
    }
}
