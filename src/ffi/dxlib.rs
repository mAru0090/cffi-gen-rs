#![allow(non_snake_case)]
use crate::utils::*;
use cffi_macro::cffi;
//use cffi_macro::cffi_gen;
//use cffi_macro::cffi_module;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DxLibError {
    #[error("Failed to DxLib_Init()")]
    InitializeError,
    #[error("Failed to DxLib_End()")]
    FinalizeError,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// =======================================================
// cffi_gen! {
//  config{
//      [config_attribute*]
//  }
//  functions{
//      [signature*]
//  }
// }
// =======================================================

// ライブラリ名
cffi! {
    config{
        #[library_name = "DxLib_x64"], // ライブラリ名
        #[link_type = "dylib"], // リンクタイプ
        #[as_result], // 関数戻り値をanyhow::Resultに変換
        #[arg_convert = default], // 関数引数の変換処理をデフォルトにする
        #[func_name_top_prefix = "dx_" ], // ffi関数生成関数の最初にdx_をつけて生成
    }
    functions{
        // ライブラリの初期化
        //#[alias = "dxlib_init"]
        //#[as_result] // 関数戻り値をanyhow::Resultに変換
        fn DxLib_Init() -> i32;
        // ライブラリ使用の終了関数
        fn DxLib_End() -> i32;
        #[error_condition = "result == i32::MAX"]
        fn WaitKey() -> i32;
        #[error_condition = "result == i32::MAX"]
        fn GetColor(red:i32,green:i32,blue:i32)->i32;
        fn ChangeWindowMode(mode: i32) -> i32;
        fn SetUseCharCodeFormat(char_code_format: i32) -> i32;
        fn DrawString(x:i32,y:i32,string:&String,color:i32) -> i32;
        fn TestFunc(p: &String) -> i32;
        //fn TestFunc2(#[option_default = "0"]p: Option<i32>) -> i32,
        fn TestFunc2(s: &str) -> i32;
        //fn TestFunc3(s: &mut Vec<std::os::raw::c_char>) -> i32;
    }
}

/*
cffi!{
    fn dx_DxLib_Init() -> i32;
    mod runtime {
        fn dx_DxLib_Init() -> i32;
    }
}
*/
