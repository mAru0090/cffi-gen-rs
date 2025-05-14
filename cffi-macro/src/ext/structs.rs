use syn::Attribute;
use syn::Item;
use syn::ItemFn;
use syn::ItemMod;
use syn::ItemStruct;
use syn::ItemImpl;
use syn::Signature;
use syn::Token;
use syn::punctuated::Punctuated;
// 属性付き関数
pub struct FunctionWithAttrs {
    pub attrs: Vec<Attribute>,
    pub sig: Signature,
}
// 属性つきインラインモジュール
pub struct ModWithAttrs {
    pub item_mod: ItemMod,
    pub fns: Vec<FunctionWithAttrs>,
}
// 属性つき構造体
pub struct StructWithAttrs{
    pub item_struct: ItemStruct,
}
// 属性つき構造体実装
pub struct StructImplWithAttrs{
    pub item_struct_impl: ItemImpl,
}

// cffi入力タイプ
pub enum CFFIInputType {
    Fn(Punctuated<FunctionWithAttrs, Token![;]>),
    Mod(Vec<ModWithAttrs>),
}

/* TODO: 下記に要変更
// cffiメイン入力
pub struct CFFIInput {
    pub config_attrs: Vec<Attribute>,
    pub inputs: Vec<CFFIInputType>,
}
*/

// cffiメイン入力
pub struct CFFIInput {
    pub config_attrs: Vec<Attribute>,
    pub fns: Punctuated<FunctionWithAttrs, Token![;]>,
}



