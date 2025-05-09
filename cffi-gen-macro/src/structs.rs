use syn::Token;
use syn::Signature;
use syn::Attribute;
use syn::punctuated::Punctuated;
// 属性付き関数
pub struct FunctionWithAttrs {
    pub attrs: Vec<Attribute>,
    pub sig: Signature,
}
// マクロ全体
pub struct CFFIGenInput {
    pub config_attrs: Vec<Attribute>,
    pub fns: Punctuated<FunctionWithAttrs, Token![,]>,
}

