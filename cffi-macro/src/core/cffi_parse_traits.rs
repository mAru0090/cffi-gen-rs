use crate::ext::structs::*;
use anyhow::Result;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::Mutex;
use syn::Attribute;
use syn::ForeignItem;
use syn::ItemForeignMod;
use syn::{
    Expr, ExprLit, FnArg, GenericArgument, Ident, Item, ItemFn, ItemMod, Lit, LitStr, Meta,
    MetaNameValue, Pat, PatType, PathArguments, ReturnType, Signature, Token, Type, TypeParamBound,
    TypePath, TypeReference, braced,
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

impl Parse for CFFIInput {
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

        let fns = Punctuated::<FunctionWithAttrs, Token![;]>::parse_terminated(&content_functions)?;
        Ok(CFFIInput {
            config_attrs: config_attrs,
            fns,
        })
    }
}
/*
impl Parse for CFFIModuleInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // モジュールをパース
        let module: ItemMod = input.parse()?;

        //println!("{}", quote! { #module });
        // `attrs` は module の属性として取得
        let config_attrs = module.clone().attrs;

        let mut fns = Punctuated::new();

        if let Some((_, items)) = module.content {
            for item in items {
                match item {
                    Item::Verbatim(ts) => {}
                    _ => {
                        return Err(syn::Error::new_spanned(
                            item,
                            "Only function declarations are allowed inside the module.",
                        ));
                    }
                }
            }
        } else {
            return Err(syn::Error::new_spanned(
                module,
                "Expected inline module body (not an external mod).",
            ));
        }

        Ok(CFFIModuleInput { config_attrs, fns })
    }
}
*/
