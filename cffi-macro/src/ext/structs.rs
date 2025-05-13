use syn::Attribute;
use syn::Item;
use syn::ItemFn;
use syn::ItemMod;
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
// cffi入力タイプ
pub enum CFFIInputType {
    Fn(Punctuated<FunctionWithAttrs, Token![;]>),
    Mod(Vec<ModWithAttrs>),
}

// cffiメイン入力
pub struct CFFIInput {
    pub config_attrs: Vec<Attribute>,
    pub fns: Punctuated<FunctionWithAttrs, Token![;]>,
}
