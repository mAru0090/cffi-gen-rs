#![allow(non_snake_case)]
use crate::utils::*;
use cffi_gen_macro::cffi_gen;
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
cffi_gen! {
    config{
        #[library_name = "DxLib_x64"], // ライブラリ名
        //#[link_type = "dylib"], // リンクタイプ
        #[as_result], // 関数戻り値をanyhow::Resultに変換
        #[arg_convert = default], // 関数引数の変換処理をデフォルトにする
        #[func_name_top_prefix = "dx_" ], // ffi関数生成関数の最初にdx_をつけて生成
    }
    functions{
        // ライブラリの初期化
        //#[alias = "dxlib_init"]
        //#[as_result] // 関数戻り値をanyhow::Resultに変換
        fn DxLib_Init() -> i32,
        // ライブラリ使用の終了関数
        fn DxLib_End() -> i32,
        fn ChangeWindowMode(mode: i32) -> i32,
        fn TestFunc(p: impl AsRef<str>) -> i32,
    }
}
