
extern crate proc_macro;
use anyhow::Result;
use crate::cffi_analyzer::*;
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

// =====================================================================
// cffi_moduleアトリビュートでの実装処理をする関数
// =====================================================================
pub fn generate_cffi_module(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    quote!{
    }.into()
}
