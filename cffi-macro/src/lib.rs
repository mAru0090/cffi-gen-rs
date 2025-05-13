// src/lib.rs
mod core {
    pub mod cffi_analyzer;
    pub mod cffi_attribute_analyzer;
    pub mod cffi_impl;
    pub mod cffi_parse_traits;
    pub mod cffi_quote;
}
mod ext {
    pub mod defines;
    pub mod structs;
}

use crate::core::cffi_analyzer::*;
use crate::core::cffi_parse_traits::*;
use crate::ext::structs::*;
use anyhow::Result;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::Mutex;
use syn::{
    Attribute, Expr, ExprLit, FnArg, GenericArgument, Ident, Lit, LitStr, Meta, MetaNameValue,
    PathArguments, ReturnType, Signature, Token, TypeParamBound, TypePath, TypeReference, braced,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
};

/*#[proc_macro]*/
/*pub fn cffi_gen(input: TokenStream) -> TokenStream {*/
/*cffi_gen_impl::generate_cffi_gen(input)*/
/*}*/

/*#[proc_macro_attribute]*/
/*pub fn cffi_module(attr: TokenStream, item: TokenStream) -> TokenStream {*/
/*cffi_module_impl::generate_cffi_module(attr, item)*/
/*}*/
#[proc_macro]
pub fn cffi(input: TokenStream) -> TokenStream {
    crate::core::cffi_impl::generate_cffi(input)
}
