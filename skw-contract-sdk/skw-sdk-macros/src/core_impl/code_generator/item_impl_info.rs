use crate::ItemImplInfo;
use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;

impl ItemImplInfo {
    /// Generate the code that wraps
    pub fn wrapper_code(&self) -> TokenStream2 {
        let mut res = TokenStream2::new();
        for method in &self.methods {
            if method.is_public || self.is_trait_impl {
                res.extend(method.method_wrapper());
            }
        }
        res
    }

    pub fn marshall_code(&self) -> TokenStream2 {
        use quote::{format_ident, quote, ToTokens};
        let orig_name = self.ty.clone().into_token_stream();
        let mut name = quote! {Contract};
        if let Ok(input) = syn::parse::<Ident>(orig_name.into()) {
            let new_name = format_ident!("{}Contract", input);
            name = quote! {#new_name};
        };
        let mut res = TokenStream2::new();
        for method in &self.methods {
            if method.is_public || self.is_trait_impl {
                res.extend(method.marshal_method());
            }
        }
        quote! {
         #[cfg(not(target_arch = "wasm32"))]
         impl #name {
           #res
         }
        }
    }
}
// Rustfmt removes comas.
#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use syn::{Type, ImplItemMethod, parse_quote};
    use quote::quote;
    use crate::core_impl::info_extractor::ImplItemMethodInfo;


    #[test]
    fn trait_implt() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("fn method(&self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method();
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn no_args_no_return_no_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method();
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn owned_no_args_no_return_no_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method();
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }


    #[test]
    fn mut_owned_no_args_no_return() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method();
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn no_args_no_return_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::attached_deposit() != 0 {
                    skw_contract_sdk::env::panic_str("Method method doesn't accept deposit");
                }
                let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method();
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn arg_no_return_no_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self, k: u64) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    k: u64,
                }
                let Input { k, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(k, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_no_return_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&mut self, k: u64, m: Bar) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub extern "C" fn method() {
                    skw_contract_sdk::env::setup_panic_hook();
                    if skw_contract_sdk::env::attached_deposit() != 0 {
                        skw_contract_sdk::env::panic_str("Method method doesn't accept deposit");
                    }
                    #[derive(skw_contract_sdk :: serde :: Deserialize)]
                    #[serde(crate = "skw_contract_sdk::serde")]
                    struct Input {
                        k: u64,
                        m: Bar,
                    }
                    let Input { k, m, }: Input = skw_contract_sdk::serde_json::from_slice(
                        &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                    )
                    .expect("Failed to deserialize input from JSON.");
                    let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                    contract.method(k, m, );
                    skw_contract_sdk::env::state_write(&contract);
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_return_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&mut self, k: u64, m: Bar) -> Option<u64> { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub extern "C" fn method() {
                    skw_contract_sdk::env::setup_panic_hook();
                    if skw_contract_sdk::env::attached_deposit() != 0 {
                        skw_contract_sdk::env::panic_str("Method method doesn't accept deposit");
                    }
                    #[derive(skw_contract_sdk :: serde :: Deserialize)]
                    #[serde(crate = "skw_contract_sdk::serde")]
                    struct Input {
                        k: u64,
                        m: Bar,
                    }
                    let Input { k, m, }: Input = skw_contract_sdk::serde_json::from_slice(
                        &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                    )
                    .expect("Failed to deserialize input from JSON.");
                    let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                    let result = contract.method(k, m, );
                    let result =
                        skw_contract_sdk::serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                    skw_contract_sdk::env::value_return(&result);
                    skw_contract_sdk::env::state_write(&contract);
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_return_ref() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&self) -> &Option<u64> { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                let result = contract.method();
                let result =
                    skw_contract_sdk::serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                skw_contract_sdk::env::value_return(&result);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn arg_ref() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self, k: &u64) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub extern "C" fn method() {
                    skw_contract_sdk::env::setup_panic_hook();
                    #[derive(skw_contract_sdk :: serde :: Deserialize)]
                    #[serde(crate = "skw_contract_sdk::serde")]
                    struct Input {
                        k: u64,
                    }
                    let Input { k, }: Input = skw_contract_sdk::serde_json::from_slice(
                        &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                    )
                    .expect("Failed to deserialize input from JSON.");
                    let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                    contract.method(&k, );
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn arg_mut_ref() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&self, k: &mut u64) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(&mut k, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback_unwrap] x: &mut u64, y: String, #[callback_unwrap] z: Vec<u8>) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::current_account_id() != skw_contract_sdk::env::predecessor_account_id() {
                    skw_contract_sdk::env::panic_str("Method method is private");
                }
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    y: String,
                }
                let Input { y, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let data: Vec<u8> = match skw_contract_sdk::env::promise_result(0u64) {
                    skw_contract_sdk::PromiseResult::Successful(x) => x,
                    _ => skw_contract_sdk::env::panic_str("Callback computation 0 was not successful")
                };
                let mut x: u64 =
                    skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let data: Vec<u8> = match skw_contract_sdk::env::promise_result(1u64) {
                    skw_contract_sdk::PromiseResult::Successful(x) => x,
                    _ => skw_contract_sdk::env::panic_str("Callback computation 1 was not successful")
                };
                let z: Vec<u8> =
                    skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, z, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_only() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback_unwrap] x: &mut u64, #[callback_unwrap] y: String) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::current_account_id() != skw_contract_sdk::env::predecessor_account_id() {
                    skw_contract_sdk::env::panic_str("Method method is private");
                }
                let data: Vec<u8> = match skw_contract_sdk::env::promise_result(0u64) {
                    skw_contract_sdk::PromiseResult::Successful(x) => x,
                    _ => skw_contract_sdk::env::panic_str("Callback computation 0 was not successful")
                };
                let mut x: u64 =
                    skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let data: Vec<u8> = match skw_contract_sdk::env::promise_result(1u64) {
                    skw_contract_sdk::PromiseResult::Successful(x) => x,
                    _ => skw_contract_sdk::env::panic_str("Callback computation 1 was not successful")
                };
                let y: String =
                    skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, );
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_results() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback_result] x: &mut Result<u64, PromiseError>, #[callback_result] y: Result<String, PromiseError>) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::current_account_id() != skw_contract_sdk::env::predecessor_account_id() {
                    skw_contract_sdk::env::panic_str("Method method is private");
                }
                let mut x: Result<u64, PromiseError> = match skw_contract_sdk::env::promise_result(0u64) {
                    skw_contract_sdk::PromiseResult::Successful(data) => Ok(skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON")),
                    skw_contract_sdk::PromiseResult::NotReady => Err(skw_contract_sdk::PromiseError::NotReady),
                    skw_contract_sdk::PromiseResult::Failed => Err(skw_contract_sdk::PromiseError::Failed),
                };
                let y: Result<String, PromiseError> = match skw_contract_sdk::env::promise_result(1u64) {
                    skw_contract_sdk::PromiseResult::Successful(data) => Ok(skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON")),
                    skw_contract_sdk::PromiseResult::NotReady => Err(skw_contract_sdk::PromiseError::NotReady),
                    skw_contract_sdk::PromiseResult::Failed => Err(skw_contract_sdk::PromiseError::Failed),
                };
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, );
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_vec() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback_vec] x: Vec<String>, y: String) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::current_account_id() != skw_contract_sdk::env::predecessor_account_id() {
                    skw_contract_sdk::env::panic_str("Method method is private");
                }
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    y: String,
                }
                let Input { y, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let x: Vec<String> = (0..skw_contract_sdk::env::promise_results_count())
                    .map(|i| {
                        let data: Vec<u8> = match skw_contract_sdk::env::promise_result(i) {
                            skw_contract_sdk::PromiseResult::Successful(x) => x,
                            _ => skw_contract_sdk::env::panic_str(&format!("Callback computation {} was not successful", i)),
                        };
                        skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON")
                    })
                    .collect();
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(x, y, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn simple_init() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[init]
            pub fn method(k: &mut u64) -> Self { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::attached_deposit() != 0 {
                    skw_contract_sdk::env::panic_str("Method method doesn't accept deposit");
                }
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                if skw_contract_sdk::env::state_exists() {
                    skw_contract_sdk::env::panic_str("The contract has already been initialized");
                }
                let contract = Hello::method(&mut k,);
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn init_no_return() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[init]
            pub fn method(k: &mut u64) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            compile_error! { "Init methods must return the contract state" }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn init_ignore_state() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[init(ignore_state)]
            pub fn method(k: &mut u64) -> Self { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::attached_deposit() != 0 {
                    skw_contract_sdk::env::panic_str("Method method doesn't accept deposit");
                }
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract = Hello::method(&mut k,);
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn init_payable() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[init]
            #[payable]
            pub fn method(k: &mut u64) -> Self { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                #[derive(skw_contract_sdk :: serde :: Deserialize)]
                #[serde(crate = "skw_contract_sdk::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = skw_contract_sdk::serde_json::from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                if skw_contract_sdk::env::state_exists() {
                    skw_contract_sdk::env::panic_str("The contract has already been initialized");
                }
                let contract = Hello::method(&mut k,);
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_return_mut_borsh() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[result_serializer(borsh)]
            pub fn method(&mut self, #[serializer(borsh)] k: u64, #[serializer(borsh)]m: Bar) -> Option<u64> { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::attached_deposit() != 0 {
                    skw_contract_sdk::env::panic_str("Method method doesn't accept deposit");
                }
                #[derive(skw_contract_sdk :: borsh :: BorshDeserialize)]
                struct Input {
                    k: u64,
                    m: Bar,
                }
                let Input { k, m, }: Input = skw_contract_sdk::borsh::BorshDeserialize::try_from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from Borsh.");
                let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                let result = contract.method(k, m, );
                let result = skw_contract_sdk::borsh::BorshSerialize::try_to_vec(&result)
                    .expect("Failed to serialize the return value using Borsh.");
                skw_contract_sdk::env::value_return(&result);
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_mixed_serialization() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback_unwrap] #[serializer(borsh)] x: &mut u64, #[serializer(borsh)] y: String, #[callback_unwrap] #[serializer(json)] z: Vec<u8>) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::current_account_id() != skw_contract_sdk::env::predecessor_account_id() {
                    skw_contract_sdk::env::panic_str("Method method is private");
                }
                #[derive(skw_contract_sdk :: borsh :: BorshDeserialize)]
                struct Input {
                    y: String,
                }
                let Input { y, }: Input = skw_contract_sdk::borsh::BorshDeserialize::try_from_slice(
                    &skw_contract_sdk::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from Borsh.");
                let data: Vec<u8> = match skw_contract_sdk::env::promise_result(0u64) {
                    skw_contract_sdk::PromiseResult::Successful(x) => x,
                    _ => skw_contract_sdk::env::panic_str("Callback computation 0 was not successful")
                };
                let mut x: u64 = skw_contract_sdk::borsh::BorshDeserialize::try_from_slice(&data)
                    .expect("Failed to deserialize callback using Borsh");
                let data: Vec<u8> = match skw_contract_sdk::env::promise_result(1u64) {
                    skw_contract_sdk::PromiseResult::Successful(x) => x,
                    _ => skw_contract_sdk::env::panic_str("Callback computation 1 was not successful")
                };
                let z: Vec<u8> =
                    skw_contract_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, z, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn no_args_no_return_mut_payable() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("#[payable] pub fn method(&mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                skw_contract_sdk::env::setup_panic_hook();
                let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.method();
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn private_method() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("#[private] pub fn private_method(&mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn private_method() {
                skw_contract_sdk::env::setup_panic_hook();
                if skw_contract_sdk::env::current_account_id() != skw_contract_sdk::env::predecessor_account_id() {
                    skw_contract_sdk::env::panic_str("Method private_method is private");
                }
                if skw_contract_sdk::env::attached_deposit() != 0 {
                    skw_contract_sdk::env::panic_str("Method private_method doesn't accept deposit");
                }
                let mut contract: Hello = skw_contract_sdk::env::state_read().unwrap_or_default();
                contract.private_method();
                skw_contract_sdk::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn marshall_one_arg() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self, k: String) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.marshal_method();
        let expected = quote!(
                #[cfg(not(target_arch = "wasm32"))]
                pub fn method(&self, k: String,) -> skw_contract_sdk::PendingContractTx {
                  let args = skw_contract_sdk::serde_json::json!({ "k": k })
                  .to_string()
                  .into_bytes();
                  skw_contract_sdk::PendingContractTx::new_from_bytes(self.account_id.clone(), "method", args, true)
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn marshall_borsh() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str(r#"
          pub fn borsh_test(&mut self, #[serializer(borsh)] a: String) {}
        "#).unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.marshal_method();
        let expected = quote!(
                #[cfg(not(target_arch = "wasm32"))]
                pub fn borsh_test(&self, a: String,) -> skw_contract_sdk::PendingContractTx {
                  #[derive(skw_contract_sdk :: borsh :: BorshSerialize)]
                  struct Input {
                      a: String,
                  }
                  let args = Input { a, };
                  let args = skw_contract_sdk::borsh::BorshSerialize::try_to_vec(&args)
                      .expect("Failed to serialize the cross contract args using Borsh.");
                  skw_contract_sdk::PendingContractTx::new_from_bytes(self.account_id.clone(), "borsh_test", args, false)
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }
}
