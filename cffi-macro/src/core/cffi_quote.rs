extern crate proc_macro;
use crate::core::cffi_analyzer::*;
use crate::core::cffi_attribute_analyzer::*;
use crate::ext::structs::*;
use anyhow::Result;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::Mutex;
use syn::{
    Expr, ExprLit, FnArg, GenericArgument, Ident, Lit, LitStr, Meta, MetaNameValue, Pat, PatType,
    PathArguments, ReturnType, Signature, Token, Type, TypeParamBound, TypePath, TypeReference,
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
};

pub struct CFFIQuote;

impl CFFIQuote {
    pub fn quote_to_char_ptr_mut(ident: &Ident, ty: &Type) -> TokenStream {
        // 変換結果 (RawPointer) 内のCStringを保持するための識別子
        let raw_holder_ident = format_ident!("__{}_holder", ident);
        // 変換結果（RawPointer）を保持するための識別子
        let raw_ptr_ident = format_ident!("__{}_ptr", ident);
        let keep_ident = format_ident!("__{}_keep", ident);
        let default_output = quote! {
            let #raw_holder_ident = #ident.to_general_type_mut();
            let (#raw_ptr_ident,  #keep_ident) = match #raw_holder_ident {
                GeneralRawType::CharPointer(RawPointer::Mutable { ptr, owner, .. }) => {
                    (ptr, owner)
                }
                _ => panic!("Expected a valid CharPointer"),
            };
            let #ident:*mut c_char = #raw_ptr_ident;
        }
        .into();
        default_output
    }

    pub fn quote_to_char_ptr_const(ident: &Ident, ty: &Type) -> TokenStream {
        // 変換結果 (RawPointer) 内のCStringを保持するための識別子
        let raw_holder_ident = format_ident!("__{}_holder", ident);
        // 変換結果（RawPointer）を保持するための識別子
        let raw_ptr_ident = format_ident!("__{}_ptr", ident);
        let keep_ident = format_ident!("__{}_keep", ident);
        let default_output = quote! {
            let #raw_holder_ident = ToGeneralRawType::to_general_type_ref(#ident);
            let (#raw_ptr_ident,  #keep_ident) = match #raw_holder_ident {
                GeneralRawType::CharPointer(RawPointer::Constant { ptr, owner, .. }) => {
                    (ptr, owner)
                }
                _ => panic!("Expected a valid CharPointer"),
            };
            let #ident:*const c_char = #raw_ptr_ident;
        }
        .into();
        let str_output = quote! {
            let #raw_holder_ident = ToGeneralRawType::to_general_type_ref(&#ident);
            let (#raw_ptr_ident,  #keep_ident) = match #raw_holder_ident {
                GeneralRawType::CharPointer(RawPointer::Constant { ptr, owner, .. }) => {
                    (ptr, owner)
                }
                _ => panic!("Expected a valid CharPointer"),
            };
            let #ident:*const c_char = #raw_ptr_ident;
        }
        .into();
        if let Some(elem) = CFFIAnalyzer::extract_ref(&ty) {
            if let Some(path) = CFFIAnalyzer::extract_path(elem) {
                if path.is_ident("str") {
                    return str_output;
                } else {
                    return default_output;
                }
            } else {
                return default_output;
            }
        } else {
            return default_output;
        }
    }
}
