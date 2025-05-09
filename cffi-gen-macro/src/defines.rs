// =====================================================================
// =====================================================================
// マクロアトリビュート名等
// =====================================================================
// =====================================================================

// =====================================================================
// グローバル属性（マクロ全体または関数宣言上部で使用可能）
// =====================================================================
pub const M_ATTR_LIBRARY_NAME: &str = "library_name";
pub const M_ATTR_LIBRARY_LINK_TYPE: &str = "link_type";
pub const M_ATTR_AS_RESULT: &str = "as_result";
pub const M_ATTR_AS_RESULT_ERROR_TYPE: &str = "as_result_error_type";
pub const M_ATTR_AS_RESULT_ERROR_TYPE_TOP_PRIORITY: &str = "as_result_error_type_top_priority";
pub const M_ATTR_FUNC_NAME_TOP_PREFIX: &str = "func_name_top_prefix";
pub const M_ATTR_FUNC_NAME_DOWN_PREFIX: &str = "func_name_down_prefix";
pub const M_ATTR_ERROR_CONDITION: &str = "error_condition";
pub const M_ATTR_ARG_CONVERT: &str = "arg_convert";
pub const M_ATTR_FUNC_NAME: &str = "func_name";
pub const M_ATTR_FUNC_ALIAS: &str = "func_alias";
pub const M_ATTR_NOT_NULL_ASSERT: &str = "not_null_assert";
// =====================================================================
// 引数用属性（関数宣言内の引数に対して使用）
// =====================================================================
pub const M_ATTR_AS_ARG_TYPE: &str = "as_arg_type";
pub const M_ATTR_OPTION_DEFAULT: &str = "option_default";

// =====================================================================
// 汎用属性 (マクロ全体、関数宣言上部、引数のいずれかで使用可能)
// =====================================================================
pub const M_ATTR_ARG_CONVERT_STR: &str = "arg_convert_str";
pub const M_ATTR_ARG_CONVERT_STRING: &str = "arg_convert_string";
pub const M_ATTR_ARG_CONVERT_VEC_C_CHAR: &str = "arg_convert_vec_c_char";
pub const M_ATTR_ARG_CONVERT_AS_REF_STR: &str = "arg_convert_as_ref_str";
