use crate::core_impl::{
    info_extractor::{InputStructType, SerializerType, TraitItemMethodInfo},
    AttrSigInfo,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

impl TraitItemMethodInfo {
    /// Generate code that wraps the method.
    pub fn method_wrapper(&self) -> TokenStream2 {
        let ident = &self.attr_sig_info.ident;
        let ident_byte_str = &self.ident_byte_str;
        let pat_type_list = self.attr_sig_info.pat_type_list();
        let serialize = TraitItemMethodInfo::generate_serialier(
            &self.attr_sig_info,
            &self.attr_sig_info.result_serializer,
        );
        quote! {
            pub fn #ident(#pat_type_list __account_id: AccountId, __balance: skw_contract_sdk::Balance, __gas: skw_contract_sdk::Gas) -> skw_contract_sdk::Promise {
                #serialize
                skw_contract_sdk::Promise::new(__account_id)
                .function_call(
                    #ident_byte_str.to_string(),
                    args,
                    __balance,
                    __gas,
                )
            }
        }
    }

    pub fn generate_serialier(
        attr_sig_info: &AttrSigInfo,
        serializer: &SerializerType,
    ) -> TokenStream2 {
        let has_input_args = attr_sig_info.input_args().next().is_some();
        if !has_input_args {
            return quote! { let args = vec![]; };
        }
        let struct_decl = attr_sig_info.input_struct(InputStructType::Serialization);
        let constructor_call = attr_sig_info.constructor_expr();
        let constructor = quote! { let args = #constructor_call; };
        let value_ser = match serializer {
            SerializerType::JSON => quote! {
                let args = skw_contract_sdk::serde_json::to_vec(&args).expect("Failed to serialize the cross contract args using JSON.");
            },
            SerializerType::Borsh => quote! {
                let args = skw_contract_sdk::borsh::BorshSerialize::try_to_vec(&args).expect("Failed to serialize the cross contract args using Borsh.");
            },
        };

        quote! {
          #struct_decl
          #constructor
          #value_ser
        }
    }
}
