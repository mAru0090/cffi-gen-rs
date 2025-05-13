# Rust Procedural Macro:汎用C FFIラッパー自動生成ツール

このプロジェクトは、C言語ライブラリをRustから安全かつ直感的に扱えるようにするためのプロシージャルマクロです。  
複雑なFFI定義を属性ベースで簡潔に記述し、モジュール単位でRustラッパーを自動生成します。

## 特徴

- `#[cffi_module(...)]` により、C関数群を役割別モジュールとして整理・定義
- `#[cffi!{}]` 内で関数宣言やモジュールを書くだけで、FFI定義とラッパーコードを自動生成
- `arg_convert=default` による Rustネイティブ型（例: `&str`, `Vec<T>`）からC型への自動変換
- `arg_convert=false` または `arg_convert=`がない場合、`user_arg_convert(i32,i64,String,impl AsRef<T>)` 等のようにし、
   その後規定のトレイト実装を書くことで、自作の引数変換処理も可能になります(ポインタへの変換処理のみ)
- `as_result` オプションにより、戻り値を Rust の `Result` に変換し、エラー処理を簡潔化
- `func_name_top_prefix` で、C関数名のプレフィックスを省略し読みやすく
- 任意の共有ライブラリ（lib="..."）を対象とでき、ゲームライブラリやシステムAPIなど広範に対応

## 使用例

```rust
#[
    #cffi(
        lib = "MyCLib",
        as_result,
        arg_convert = default,
        func_name_top_prefix = "my_"
    )
)]
cffi!{
    mod image {
            fn Init() -> i32;
            fn LoadImage(path: AsRef<str>) -> i32;
            fn DrawImage(id: i32, x: i32, y: i32) -> i32;
        }
    }
}
```

```rust
fn main() -> anyhow::Result<()> {
    image::Init()?; 
    let id = image::LoadImage("logo.png")?;
    image::DrawImage(id, 100, 100)?;
    Ok(())
}
```

## 応用例

関数群をモジュールごとに自動整理し、DirectX9等の大規模ライブラリを責務分離した形で取り扱うことができます。  
この仕組みは、ゲームライブラリ、画像処理API、UIツールキットなど、任意のCライブラリに適用可能です。

## ライセンス

MIT
