use crate::core::cffi_analyzer::*;
use crate::ext::defines::*;
use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::{
    Attribute, Expr, ExprLit, FnArg, GenericArgument, Ident, Lit, LitStr, Meta, MetaNameValue, Pat,
    PatType, Path, PathArguments, ReturnType, Signature, Token, Type, TypeArray, TypeImplTrait,
    TypeParamBound, TypePath, TypeReference, TypeSlice,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
};

// =====================================================================
// アトリビュート用解析用構造体
// =====================================================================
pub struct CFFIAttributeAnalyzer;

impl CFFIAttributeAnalyzer {
    // =====================================================================
    // 指定アトリビュートからarg_convertを取得
    // =====================================================================
    pub fn extract_arg_convert_attr(attrs: &[syn::Attribute]) -> Option<String> {
        if let Some(lit_value) = Self::get_name_value_attr(attrs, M_ATTR_ARG_CONVERT) {
            return Some(lit_value);
        }

        // #[arg_convert = default] のようなパターンを処理
        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta {
                if path.is_ident(M_ATTR_ARG_CONVERT) {
                    if let Some(expr_path) = CFFIAnalyzer::extract_expr_path(value) {
                        // Pathが識別子で構成されていれば、それを取り出して返す
                        if let Some(ident) = expr_path.get_ident() {
                            return Some(ident.to_string());
                        } else {
                            return Some(expr_path.to_token_stream().to_string());
                        }
                    }
                }
            }
        }

        // #[arg_convert] のようなPath形式
        if let Some(path) = CFFIAnalyzer::extract_path_attr(attrs, M_ATTR_ARG_CONVERT) {
            return Some(path.to_token_stream().to_string());
        }

        None
    }

    // =====================================================================
    // 指定アトリビュートが存在するかをboolで返す関数
    // =====================================================================
    pub fn has_path_attr(attrs: &[syn::Attribute], ident: &str) -> bool {
        attrs.iter().any(|attr| attr.path().is_ident(ident))
    }

    // =====================================================================
    // 指定アトリビュートの値が存在する場合の値をStringで取得する関数
    // =====================================================================
    pub fn get_name_value_attr(attrs: &[syn::Attribute], ident: &str) -> Option<String> {
        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { value, path, .. }) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    if path.is_ident(ident) {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }

    // =====================================================================
    // 指定された属性識別子 List 型属性から `Vec<String>` を取得する。
    // =====================================================================
    pub fn get_list_strings_attr(attrs: &[Attribute], ident: &str) -> Vec<String> {
        let mut result = Vec::new();

        for attr in attrs {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(ident) {
                    result.push(meta.path.to_token_stream().to_string());
                }
                Ok(())
            });
        }

        result
    }

    // =====================================================================
    // 指定アトリビュートからas_resultが存在するかをboolで取得
    // =====================================================================
    pub fn is_as_result_attr(attrs: &[syn::Attribute]) -> bool {
        for attr in attrs {
            if let Some(_) = Self::get_name_value_attr(attrs, M_ATTR_AS_RESULT) {
                return true;
            }
            if Self::has_path_attr(attrs, M_ATTR_AS_RESULT) {
                return true;
            }
        }
        false
    }

    // =====================================================================
    // 指定アトリビュートからトッププレフィックスを取得
    // =====================================================================
    pub fn func_name_top_prefix_attr(attrs: &[syn::Attribute]) -> Option<String> {
        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { value, path, .. }) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    if path.is_ident(M_ATTR_FUNC_NAME_TOP_PREFIX) {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }

    // =====================================================================
    // 指定アトリビュートからダウンプレフィックスを取得
    // =====================================================================
    pub fn func_name_down_prefix_attr(attrs: &[syn::Attribute]) -> Option<String> {
        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { value, path, .. }) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    if path.is_ident(M_ATTR_FUNC_NAME_DOWN_PREFIX) {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }

    // =====================================================================
    // 指定アトリビュートからライブラリ名を取得
    // =====================================================================
    pub fn extract_library_name_attr(attrs: &[syn::Attribute]) -> Option<String> {
        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { value, path, .. }) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    if path.is_ident(M_ATTR_LIBRARY_NAME) {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }

    // =====================================================================
    // 指定アトリビュートからリンクタイプを取得
    // =====================================================================
    pub fn extract_link_type_attr(attrs: &[syn::Attribute]) -> Option<String> {
        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { value, path, .. }) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    if path.is_ident(M_ATTR_LIBRARY_LINK_TYPE) {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }

    // =====================================================================
    // 指定アトリビュートからエラー条件をトークンで取得
    // =====================================================================

    pub fn extract_error_condition_attr(
        attrs: &[syn::Attribute],
    ) -> Option<proc_macro2::TokenStream> {
        for attr in attrs {
            if attr.path().is_ident("error_condition") {
                if let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = value
                    {
                        let value = lit_str.value();
                        return Some(value.parse().expect("Invalid error condition expression"));
                    }
                }
            }
        }
        None
    }

    // =====================================================================
    // 指定アトリビュートから関数エイリアス名を取得
    // =====================================================================
    pub fn extract_func_alias_attr(attrs: &[syn::Attribute]) -> Option<String> {
        for attr in attrs {
            if attr.path().is_ident("alias") {
                if let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = value
                    {
                        return Some(lit_str.value());
                    }
                }
            }
        }
        None
    }
    pub fn extract_option_default_expr_attr(
        attrs: &[syn::Attribute],
    ) -> Option<proc_macro2::TokenStream> {
        for attr in attrs {
            if attr.path().is_ident(M_ATTR_OPTION_DEFAULT) {
                if let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = value
                    {
                        let value = lit_str.value();
                        return Some(match value.as_str() {
                            "null" => quote! { std::ptr::null() },
                            "null_mut" => quote! { std::ptr::null_mut() },
                            "default" => quote! { Default::default() },
                            other => {
                                let tokens: proc_macro2::TokenStream =
                                    other.parse().expect("Invalid default literal");
                                quote! { #tokens }
                            }
                        });
                    }
                }
            }
        }
        None
    }
}
