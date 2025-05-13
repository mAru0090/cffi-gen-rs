extern crate proc_macro;
use crate::core::cffi_analyzer::*;
use crate::core::cffi_attribute_analyzer::*;
use crate::core::cffi_quote::*;
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

// =====================================================================
// cffiアトリビュートでの実装処理をする関数
// =====================================================================
pub fn generate_cffi(input: TokenStream) -> TokenStream {
    let CFFIInput { config_attrs, fns } = parse_macro_input!(input as CFFIInput);

    //let CFFIGenInput { config_attrs, fns } = parse_macro_input!(input as CFFIGenInput);

    let lib_name = CFFIAttributeAnalyzer::extract_library_name_attr(&config_attrs);
    // CString を使うための import
    let mut output = quote! {
        use std::ffi::CString;
        use std::os::raw::c_char;
    };

    let mut extern_name_top_prefix =
        if let Some(top_prefix) = CFFIAttributeAnalyzer::func_name_top_prefix_attr(&config_attrs) {
            top_prefix
        } else {
            String::new()
        };
    let mut extern_name_down_prefix = if let Some(down_prefix) =
        CFFIAttributeAnalyzer::func_name_down_prefix_attr(&config_attrs)
    {
        down_prefix
    } else {
        String::new()
    };
    let link_type =
        if let Some(link_type) = CFFIAttributeAnalyzer::extract_link_type_attr(&config_attrs) {
            link_type
        } else {
            String::new()
        };
    let mut is_as_result = CFFIAttributeAnalyzer::is_as_result_attr(&config_attrs);
    let mut arg_convert = CFFIAttributeAnalyzer::extract_arg_convert_attr(&config_attrs).unwrap();
    for FunctionWithAttrs { attrs, sig } in fns.iter() {
        let wrapper_name = CFFIAttributeAnalyzer::extract_func_alias_attr(attrs)
            .map(|alias| format_ident!("{}", alias))
            .unwrap_or_else(|| sig.ident.clone());

        // 関数単位で指定がある場合、arg_convertの値を上書き
        let fn_arg_convert =
            if let Some(fn_arg_convert) = CFFIAttributeAnalyzer::extract_arg_convert_attr(attrs) {
                fn_arg_convert.clone()
            } else {
                arg_convert.clone()
            };

        // 関数単位で指定がある場合、as_resultの値を上書き
        let is_fn_as_result = if CFFIAttributeAnalyzer::is_as_result_attr(attrs) {
            true
        } else {
            is_as_result // configの設定を引き継ぐ
        };
        // 関数単位で指定がある場合、プレフィックス指定を上書き
        if let Some(top_prefix) = CFFIAttributeAnalyzer::func_name_top_prefix_attr(attrs) {
            extern_name_top_prefix = top_prefix;
        } else if let Some(down_prefix) = CFFIAttributeAnalyzer::func_name_down_prefix_attr(attrs) {
            extern_name_down_prefix = down_prefix;
        }

        let extern_name = if !extern_name_top_prefix.is_empty() {
            format_ident!("{}{}", extern_name_top_prefix, sig.ident.clone())
        } else if !extern_name_down_prefix.is_empty() {
            format_ident!("{}{}", extern_name_down_prefix, sig.ident.clone())
        } else {
            format_ident!("{}", sig.ident.clone())
        };

        let output_ty = &sig.output;
        let generics = &sig.generics;

        let mut wrapper_args = Vec::new();
        let mut extern_args = Vec::new();
        let mut convert_stmts = Vec::new();
        let mut call_idents = Vec::new();

        let return_type = CFFIAnalyzer::get_return_type(&sig).unwrap();

        let error_condition = CFFIAttributeAnalyzer::extract_error_condition_attr(attrs)
            .unwrap_or_else(|| quote! { result as i32 == -1i32 });

        for arg in sig.inputs.iter() {
            if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg {
                let ident = match &**pat {
                    Pat::Ident(pi) => &pi.ident,
                    _ => panic!("パターン付き引数は未対応です"),
                };
                if fn_arg_convert.contains("default") || fn_arg_convert.contains("true") {
                    if let Some(inner_ty) = CFFIAnalyzer::is_option(&ty) {
                        // Option<T> の場合
                        wrapper_args.push(quote! { #ident: Option<#inner_ty> });

                        let default_expr =
                            CFFIAttributeAnalyzer::extract_option_default_expr_attr(&attrs)
                                .unwrap_or_else(|| quote! { Default::default() });

                        convert_stmts.push(quote! {
                            let #ident = match #ident {
                                Some(value) => value,
                                None => #default_expr,
                            };
                        });

                        extern_args.push(quote! { #ident: #inner_ty });
                        call_idents.push(quote! { #ident });

                        continue;
                    }

                    if CFFIAnalyzer::is_impl_as_ref_type(&ty) {
                        // まず ty 自体が参照型かどうかを判定
                        if let Some(ref_type) = CFFIAnalyzer::extract_ref(&**ty) {
                            if let Some(inner_ty) = CFFIAnalyzer::extract_as_ref_generic(&ref_type)
                            {
                                if let Some(path) = CFFIAnalyzer::extract_path(inner_ty) {
                                    let ident_str = path.segments.last().unwrap().ident.to_string();

                                    if ident_str == "str" || ident_str == "String" {
                                        wrapper_args.push(quote! {
                                            #ident: &impl AsRef<#inner_ty>
                                        });

                                        extern_args.push(quote! {
                                            #ident: *const ::std::os::raw::c_char
                                        });

                                        let holder_ident = format_ident!("__{}_holder", ident);
                                        convert_stmts.push(quote! {
                                            let #holder_ident = CStringHolder::new(#ident.as_ref());
                                            let #ident = #holder_ident.as_ptr();
                                        });

                                        call_idents.push(quote! { #ident });
                                        continue;
                                    }
                                }
                                if let Some(path) = CFFIAnalyzer::extract_path(inner_ty) {
                                    let ident_str = path.segments.last().unwrap().ident.to_string();

                                    if ident_str == "str" || ident_str == "String" {
                                        wrapper_args.push(quote! {
                                            #ident: &mut impl AsRef<#inner_ty>
                                        });

                                        extern_args.push(quote! {
                                            #ident: *const ::std::os::raw::c_char
                                        });

                                        let holder_ident = format_ident!("__{}_holder", ident);
                                        convert_stmts.push(quote! {
                                            let #holder_ident = CStringHolder::new(#ident.as_ref());
                                            let #ident = #holder_ident.as_ptr();
                                        });

                                        call_idents.push(quote! { #ident });
                                        continue;
                                    }
                                }
                            }
                        }

                        // 通常の impl AsRef<T> 型（参照ではない）
                        if let Some(inner_ty) = CFFIAnalyzer::extract_as_ref_generic(&ty) {
                            if let Some(path) = CFFIAnalyzer::extract_path(inner_ty) {
                                let ident_str = path.segments.last().unwrap().ident.to_string();
                                if ident_str == "str" || ident_str == "String" {
                                    wrapper_args.push(quote! {
                                        #ident: impl AsRef<#inner_ty>
                                    });

                                    extern_args.push(quote! {
                                        #ident: *const ::std::os::raw::c_char
                                    });

                                    let holder_ident = format_ident!("__{}_holder", ident);
                                    convert_stmts.push(quote! {
                                        let #holder_ident = CStringHolder::new(#ident.as_ref());
                                        let #ident = #holder_ident.as_ptr();
                                    });

                                    call_idents.push(quote! { #ident });
                                    continue;
                                }
                            }

                            // 汎用パターン（*const T）
                            wrapper_args.push(quote! {
                                #ident: impl AsRef<#inner_ty>
                            });

                            extern_args.push(quote! {
                                #ident: *const #inner_ty
                            });

                            convert_stmts.push(quote! {
                                let #ident = #ident.as_ref().as_ptr();
                            });

                            call_idents.push(quote! { #ident });
                            continue;
                        }
                    } else if CFFIAnalyzer::is_impl_as_mut_type(&ty) {
                        // まず `ty` 自体が参照型かどうかを判定する
                        if let Some(ref_type) = CFFIAnalyzer::extract_ref(&**ty) {
                            if let Some(inner_ty) = CFFIAnalyzer::extract_as_mut_generic(&ref_type)
                            {
                                // &mut impl AsMut<[T]> にマッチ
                                if let Type::Slice(slice) = inner_ty {
                                    let elem_ty = &slice.elem;

                                    wrapper_args.push(quote! {
                                        #ident: &mut impl AsMut<[#elem_ty]>
                                    });

                                    extern_args.push(quote! {
                                        #ident: *mut #elem_ty
                                    });

                                    convert_stmts.push(quote! {
                                        let #ident = #ident.as_mut().as_mut_ptr();
                                    });

                                    call_idents.push(quote! { #ident });
                                    continue;
                                }
                            }
                            if let Some(inner_ty) = CFFIAnalyzer::extract_as_mut_generic(&ref_type)
                            {
                                // &impl AsMut<[T]> にマッチ
                                if let Type::Slice(slice) = inner_ty {
                                    let elem_ty = &slice.elem;

                                    wrapper_args.push(quote! {
                                        #ident: &impl AsMut<[#elem_ty]>
                                    });

                                    extern_args.push(quote! {
                                        #ident: *mut #elem_ty
                                    });

                                    convert_stmts.push(quote! {
                                        let #ident = #ident.as_mut().as_mut_ptr();
                                    });

                                    call_idents.push(quote! { #ident });
                                    continue;
                                }
                            }
                        }

                        // 通常の impl AsMut<[T]> 型の処理
                        if let Some(inner_ty) = CFFIAnalyzer::extract_as_mut_generic(&ty) {
                            if let Type::Slice(slice) = inner_ty {
                                let elem_ty = &slice.elem;

                                wrapper_args.push(quote! {
                                    #ident: impl AsMut<[#elem_ty]>
                                });

                                extern_args.push(quote! {
                                    #ident: *mut #elem_ty
                                });

                                convert_stmts.push(quote! {
                                    let #ident = #ident.as_mut().as_mut_ptr();
                                });

                                call_idents.push(quote! { #ident });
                                continue;
                            }
                        }
                    }

                    if CFFIAnalyzer::is_impl_to_string(&ty) {
                        wrapper_args.push(quote! { #ident: impl ToString });
                        extern_args.push(quote! { #ident: *const c_char });

                        let holder_ident = format_ident!("__{}_holder", ident);
                        convert_stmts.push(quote! {
                            let #holder_ident = CStringHolder::new(#ident.to_string());
                            let #ident = #holder_ident.as_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    }

                    if CFFIAnalyzer::is_impl_display(&ty) {
                        wrapper_args.push(quote! { #ident: impl Display });
                        extern_args.push(quote! { #ident: *const c_char });

                        let holder_ident = format_ident!("__{}_holder", ident);
                        convert_stmts.push(quote! {
                            let #holder_ident = CStringHolder::new(#ident.to_string());
                            let #ident = #holder_ident.as_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    }

                    // `impl Into<Vec<T>>` の場合、不変と可変を分けて処理
                    if CFFIAnalyzer::is_impl_trait_into_vec(&ty) {
                        // Vec<T> の T を取得
                        let inner_ty = CFFIAnalyzer::extract_vec_inner_type_from_impl_trait(&ty);

                        if let Some(inner_ty) = inner_ty {
                            // 不変Vec<T> → *const T
                            if let Some(_) = CFFIAnalyzer::is_ref_vec_type(&ty) {
                                wrapper_args.push(quote! { #ident: impl Into<Vec<#inner_ty>> });
                                extern_args.push(quote! { #ident: *const #inner_ty });

                                convert_stmts.push(quote! {
                                    let #ident = #ident.as_ptr();
                                });

                                call_idents.push(quote! { #ident });
                                continue;
                            }
                            // 可変Vec<T> → *mut T
                            else if let Some(_) = CFFIAnalyzer::is_mut_ref_vec_type(&ty) {
                                wrapper_args.push(quote! { #ident: impl Into<Vec<#inner_ty>> });
                                extern_args.push(quote! { #ident: *mut #inner_ty });

                                convert_stmts.push(quote! {
                                    let #ident = #ident.as_mut_ptr();
                                });

                                call_idents.push(quote! { #ident });
                                continue;
                            }
                        }
                    }
                    // 配列の場合は、*const Tに変換
                    if CFFIAnalyzer::is_array(&ty) {
                        let (inner_ty, n) = CFFIAnalyzer::extract_array(&ty).unwrap();
                        wrapper_args.push(quote! { #ident: [#inner_ty;#n] });
                        extern_args.push(quote! { #ident: *const #inner_ty });

                        convert_stmts.push(quote! {
                            let #ident = #ident.as_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    } else if CFFIAnalyzer::is_mut_array(&ty) {
                        let (inner_ty, n) = CFFIAnalyzer::extract_mut_array(&ty).unwrap();
                        wrapper_args.push(quote! { #ident: &mut [#inner_ty;#n] });
                        extern_args.push(quote! { #ident: *mut #inner_ty });

                        convert_stmts.push(quote! {
                            let #ident = #ident.as_mut_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    }
                    // 不変スライスの場合は、*const Tに変換
                    if CFFIAnalyzer::is_slice(&ty) {
                        let inner_ty = CFFIAnalyzer::extract_slice(&ty);
                        wrapper_args.push(quote! { #ident: &[#inner_ty] });
                        extern_args.push(quote! { #ident: *const #inner_ty });

                        convert_stmts.push(quote! {
                            let #ident = #ident.as_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    // 可変スライスの場合は、*mut Tに変換
                    } else if CFFIAnalyzer::is_mut_slice(&ty) {
                        let inner_ty = CFFIAnalyzer::extract_mut_slice(&ty);
                        wrapper_args.push(quote! { #ident: &mut [#inner_ty] });
                        extern_args.push(quote! { #ident: *mut #inner_ty });

                        convert_stmts.push(quote! {
                            let #ident = #ident.as_mut_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    }
                    // Vec<T>の場合は、*const Tに変換
                    if CFFIAnalyzer::is_vec_type(&ty) {
                        let inner_ty = CFFIAnalyzer::extract_vec_inner_type(&ty);
                        wrapper_args.push(quote! { #ident: Vec<#inner_ty> });
                        extern_args.push(quote! { #ident: *const #inner_ty });

                        convert_stmts.push(quote! {
                            let #ident = #ident.as_ptr();
                        });

                        call_idents.push(quote! { #ident });
                        continue;
                    // &mut Vec<T>の場合は、*mut Tに変換
                    } else if CFFIAnalyzer::is_mut_vec_type(&ty) {
                        let inner_ty = CFFIAnalyzer::extract_vec_inner_type(&ty);
                        wrapper_args.push(quote! { #ident: &mut Vec<#inner_ty> });
                        extern_args.push(quote! { #ident: *mut #inner_ty });
                        /*
                                                // 変換結果 (RawPointer) 内のCStringを保持するための識別子
                                                let raw_holder_ident = format_ident!("__{}_holder", ident);
                                                // 変換結果（RawPointer）を保持するための識別子
                                                let raw_ptr_ident = format_ident!("__{}_ptr", ident);

                                                convert_stmts.push(quote! {
                                                        let #raw_holder_ident = ToGeneralRawType::to_general_type_mut(#ident);
                                                        let (#raw_ptr_ident, _keep) = match #raw_holder_ident {
                                                            GeneralRawType::CharPointer(RawPointer::Mutable { ptr, owner, .. }) => {
                                                                (ptr, owner)
                                                            }
                                                            _ => panic!("Expected a valid CharPointer"),
                                                        };
                                                        let #ident = #raw_ptr_ident;
                                                    });

                        */
                        convert_stmts.push(CFFIQuote::quote_to_char_ptr_mut(&ident, &ty).into());
                        call_idents.push(quote! { #ident });
                        continue;
                    }

                    // &str の場合は *const c_char に変換
                    if let Some(elem) = CFFIAnalyzer::extract_ref(&ty) {
                        if let Some(path) = CFFIAnalyzer::extract_path(elem) {
                            if path.is_ident("str") {
                                wrapper_args.push(quote! { #ident: &str });
                                extern_args.push(quote! { #ident: *const c_char });

                                /*
                                    // 変換結果 (RawPointer) 内のCStringを保持するための識別子
                                    let raw_holder_ident = format_ident!("__{}_holder", ident);
                                    // 変換結果（RawPointer）を保持するための識別子
                                    let raw_ptr_ident = format_ident!("__{}_ptr", ident);

                                    convert_stmts.push(quote! {
                                    let #raw_holder_ident = ToGeneralRawType::to_general_type_ref(&#ident);
                                    let (#raw_ptr_ident, _keep) = match #raw_holder_ident {
                                        GeneralRawType::CharPointer(RawPointer::Constant { ptr, owner, .. }) => {
                                            (ptr, owner)
                                        }
                                        _ => panic!("Expected a valid CharPointer"),
                                    };
                                    let #ident = #raw_ptr_ident;
                                });*/

                                convert_stmts
                                    .push(CFFIQuote::quote_to_char_ptr_const(&ident, &ty).into());

                                call_idents.push(quote! { #ident });
                                continue;
                            }
                        }
                    }

                    if let Some(path) = CFFIAnalyzer::extract_path(&ty) {
                        if path.is_ident("String") {
                            wrapper_args.push(quote! { #ident: String });
                            extern_args.push(quote! { #ident: *const c_char });

                            // 変換結果 (RawPointer) 内のCStringを保持するための識別子
                            //let raw_holder_ident = format_ident!("__{}_holder", ident);
                            //// 変換結果（RawPointer）を保持するための識別子
                            //let raw_ptr_ident = format_ident!("__{}_ptr", ident);
                            /*
                                                        convert_stmts.push(quote! {
                                                            let #raw_holder_ident = ToGeneralRawType::to_general_type_owned(#ident);
                                                            let (#raw_ptr_ident, _keep) = match #raw_holder_ident {
                                                                GeneralRawType::CharPointer(RawPointer::Constant { ptr, owner, .. }) => {
                                                                    (ptr, owner)
                                                                }
                                                                _ => panic!("Expected a valid CharPointer"),
                                                            };
                                                            let #ident = #raw_ptr_ident;
                                                        });
                            */

                            convert_stmts
                                .push(CFFIQuote::quote_to_char_ptr_const(&ident, &ty).into());
                            call_idents.push(quote! { #ident });
                            continue;
                        }
                    }

                    // &String の場合は *const c_char に変換
                    if let Some(elem) = CFFIAnalyzer::extract_ref(&ty) {
                        if let Some(path) = CFFIAnalyzer::extract_path(elem) {
                            if path.is_ident("String") {
                                wrapper_args.push(quote! { #ident: &String });
                                extern_args.push(quote! { #ident: *const c_char });

                                // 変換結果 (RawPointer) 内のCStringを保持するための識別子
                                //let raw_holder_ident = format_ident!("__{}_holder", ident);
                                // 変換結果（RawPointer）を保持するための識別子
                                //let raw_ptr_ident = format_ident!("__{}_ptr", ident);
                                /*
                                                                convert_stmts.push(quote! {
                                                                let #raw_holder_ident = ToGeneralRawType::to_general_type_ref(#ident);
                                                                let (#raw_ptr_ident, _keep) = match #raw_holder_ident {
                                                                    GeneralRawType::CharPointer(RawPointer::Constant { ptr, owner, .. }) => {
                                                                        (ptr, owner)
                                                                    }
                                                                    _ => panic!("Expected a valid CharPointer"),
                                                                };
                                                                let #ident = #raw_ptr_ident;
                                                            });
                                */

                                convert_stmts
                                    .push(CFFIQuote::quote_to_char_ptr_const(&ident, &ty).into());
                                call_idents.push(quote! { #ident });
                                continue;
                            }
                        }
                    }
                    // &mut String の場合は CString に変換 (可変ポインタ *mut c_char)

                    if let Some(elem) = CFFIAnalyzer::extract_mut(&ty) {
                        if let Some(path) = CFFIAnalyzer::extract_path(elem) {
                            if path.is_ident("String") {
                                // &mut String の場合
                                wrapper_args.push(quote! { #ident: &mut String });
                                extern_args.push(quote! { #ident: *mut c_char });

                                // 変換結果 (RawPointer) 内のCStringを保持するための識別子
                                //let raw_holder_ident = format_ident!("__{}_holder", ident);
                                // 変換結果（RawPointer）を保持するための識別子
                                //let raw_ptr_ident = format_ident!("__{}_ptr", ident);

                                /*
                                    convert_stmts.push(quote! {
                                    let #raw_holder_ident = ToGeneralRawType::to_general_type_mut(#ident);
                                    let (#raw_ptr_ident, _keep) = match #raw_holder_ident {
                                        GeneralRawType::CharPointer(RawPointer::Mutable { ptr, owner, .. }) => {
                                            (ptr, owner)
                                        }
                                        _ => panic!("Expected a valid CharPointer"),
                                    };
                                    let #ident = #raw_ptr_ident;
                                });
                                    */
                                convert_stmts
                                    .push(CFFIQuote::quote_to_char_ptr_mut(&ident, &ty).into());

                                call_idents.push(quote! { #ident });
                                continue;
                            }
                        }
                    }
                    // それ以外はそのまま
                    wrapper_args.push(quote! { #ident: #ty });
                    extern_args.push(quote! { #ident: #ty });
                    call_idents.push(quote! { #ident });
                } else {
                    // それ以外はそのまま
                    wrapper_args.push(quote! { #ident: #ty });
                    extern_args.push(quote! { #ident: #ty });
                    call_idents.push(quote! { #ident });
                }
            }
        }

        let extern_block = if !link_type.is_empty() {
            quote! {
                #[link(name = #lib_name,kind=#link_type)]
                unsafe extern "stdcall" {
                    fn #extern_name(#(#extern_args),*) #output_ty;
                }
            }
        } else {
            quote! {
                #[link(name = #lib_name)]
                unsafe extern "stdcall" {
                    fn #extern_name(#(#extern_args),*) #output_ty;
                }
            }
        };

        output.extend(extern_block);
        // DxLib_Init と DxLib_End 用の処理
        if wrapper_name == "DxLib_Init" {
            if is_fn_as_result {
                let wrapper_fn = quote! {
                  pub fn #wrapper_name #generics( #(#wrapper_args),* ) -> anyhow::Result<#return_type, DxLibError> {
                    #(#convert_stmts)*
                    unsafe {
                        let result: #return_type = #extern_name(#(#call_idents),*);
                        if #error_condition {
                            return Err(DxLibError::InitializeError);
                        } else {
                            return Ok(result);
                        }
                    }
                    }
                };
                output.extend(wrapper_fn);
            } else {
                let wrapper_fn = quote! {
                  pub fn #wrapper_name #generics( #(#wrapper_args),* ) -> #return_type {
                    #(#convert_stmts)*
                    unsafe {
                        let result: #return_type = #extern_name(#(#call_idents),*);
                        if #error_condition {
                            return -1;
                        } else {
                            return result;
                        }
                    }
                    }
                };
                output.extend(wrapper_fn);
            }
            continue; // 次の関数の処理に進む
        }
        if wrapper_name == "DxLib_End" {
            if is_fn_as_result {
                let wrapper_fn = quote! {
                  pub fn #wrapper_name #generics( #(#wrapper_args),* ) -> anyhow::Result<#return_type, DxLibError> {
                    #(#convert_stmts)*

                    unsafe {
                        let result: #return_type = #extern_name(#(#call_idents),*);

                        if #error_condition {
                            return Err(DxLibError::FinalizeError);
                        } else {
                            return Ok(result);
                        }

                     }
                    }
                };
                output.extend(wrapper_fn);
            } else {
                let wrapper_fn = quote! {
                  pub fn #wrapper_name #generics( #(#wrapper_args),* ) -> #return_type {
                    #(#convert_stmts)*
                    unsafe {
                        let result: #return_type = #extern_name(#(#call_idents),*);
                        if #error_condition {
                            return -1;
                        } else {
                            return result;
                        }
                    }
                    }
                };
                output.extend(wrapper_fn);
            }

            continue; // 次の関数の処理に進む
        }
        if is_fn_as_result {
            // wrapper 関数の生成
            let wrapper_fn = quote! {
                pub fn #wrapper_name #generics( #(#wrapper_args),* ) -> anyhow::Result<#return_type, DxLibError> {
                    #(#convert_stmts)*

                    unsafe {
                        let result: #return_type = #extern_name(#(#call_idents),*);
                        if #error_condition {
                            return Err(DxLibError::Other(anyhow::anyhow!("Error in {}", stringify!(#wrapper_name))));
                        } else {
                            return Ok(result);
                        }
                    }
                }
            };

            output.extend(wrapper_fn);
        } else {
            // wrapper 関数の生成
            let wrapper_fn = quote! {
                pub fn #wrapper_name #generics( #(#wrapper_args),* ) -> #return_type {
                    #(#convert_stmts)*

                    unsafe {
                        let result: #return_type = #extern_name(#(#call_idents),*);
                        if #error_condition {
                            return -1;
                        } else {
                            return result;
                        }
                    }
                }
            };

            output.extend(wrapper_fn);
        }
    }

    TokenStream::from(output)
}
