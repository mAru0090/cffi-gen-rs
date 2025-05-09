// src/lib.rs
mod cffi_attribute_analyzer;
mod cffi_analyzer;
mod cffi_error;
mod cffi_gen_impl;
mod cffi_impl;
mod cffi_module_impl;
mod defines;
mod structs;

use anyhow::Result;
use cffi_analyzer::*;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::Mutex;
use structs::*;
use syn::{
    Attribute, Expr, ExprLit, FnArg, GenericArgument, Ident, Lit, LitStr, Meta, MetaNameValue,
    PathArguments, ReturnType, Signature, Token, TypeParamBound, TypePath, TypeReference, braced,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
};

impl Parse for FunctionWithAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let sig: Signature = input.parse()?;
        Ok(FunctionWithAttrs { attrs, sig })
    }
}

impl Parse for CFFIGenInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // `config` ブロック
        input.parse::<Ident>().and_then(|ident| {
            if ident != "config" {
                return Err(syn::Error::new(ident.span(), "expected `config`"));
            }
            Ok(())
        })?;

        let content_config;
        braced!(content_config in input);
        //let config_attrs = content_config.call(syn::Attribute::parse_outer)?;

        // ←ここで自前ループ
        let mut config_attrs = Vec::new();
        while !content_config.is_empty() {
            // Attribute::parse_outer は Vec<Attribute> を返す
            let mut attrs = content_config.call(syn::Attribute::parse_outer)?;
            config_attrs.append(&mut attrs);

            // カンマがあれば消費して次へ
            if content_config.peek(Token![,]) {
                content_config.parse::<Token![,]>()?;
            }
        }
        // `functions` ブロック
        input.parse::<Ident>().and_then(|ident| {
            if ident != "functions" {
                return Err(syn::Error::new(ident.span(), "expected `functions`"));
            }
            Ok(())
        })?;

        let content_functions;
        braced!(content_functions in input);

        let fns = Punctuated::<FunctionWithAttrs, Token![,]>::parse_terminated(&content_functions)?;
        Ok(CFFIGenInput {
            config_attrs: config_attrs,
            fns,
        })
    }
}
#[proc_macro]
pub fn cffi_gen(input: TokenStream) -> TokenStream {
    cffi_gen_impl::generate_cffi_gen(input)
}

#[proc_macro_attribute]
pub fn cffi_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    cffi_module_impl::generate_cffi_module(attr, item)
}
#[proc_macro_attribute]
pub fn cffi(attr: TokenStream, item: TokenStream) -> TokenStream {
    cffi_impl::generate_cffi(attr, item)
}
