use std::ffi::CString;
use std::os::raw::c_char;
use std::{any::Any, marker::PhantomData};
pub struct CStringHolder {
    _c_string: CString,
    ptr: *const c_char,
}

impl CStringHolder {
    pub fn new(s: impl ToString) -> Self {
        let c_string = CString::new(s.to_string()).unwrap();
        Self {
            _c_string: c_string,
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn as_ptr(&self) -> *const std::os::raw::c_char {
        self._c_string.as_ptr()
    }
}

/*
// ポインター型のラップ(値,保持者)
pub enum RawPointer<T> {
    Mutable(*mut T, Option<Box<dyn std::any::Any>>),
    Constant(*const T, Option<Box<dyn std::any::Any>>),
}
*/

pub enum RawPointer<'a,T> {
    Mutable {
        ptr: *mut T,
        owner: Option<Box<dyn Any>>,
        _marker: PhantomData<&'a T>,
    },
    Constant {
        ptr: *const T,
        owner: Option<Box<dyn Any>>,
        _marker: PhantomData<&'a T>,
    },
}
impl<'a,T> RawPointer<'a,T> {
    pub fn new_mut(ptr: *mut T, owner: Option<Box<dyn Any>>) -> Self {
        RawPointer::Mutable {
            ptr,
            owner,
            _marker: PhantomData,
        }
    }

    pub fn new_const(ptr: *const T, owner: Option<Box<dyn Any>>) -> Self {
        RawPointer::Constant {
            ptr,
            owner,
            _marker: PhantomData,
        }
    }
}
pub enum GeneralType<'a> {
    Int(std::os::raw::c_int),
    Float(std::os::raw::c_float),
    Double(std::os::raw::c_double),
    //CharPointer(RawPointer<std::os::raw::c_char>),
    //VoidPointer(RawPointer<std::os::raw::c_void>),
    CharPointer(RawPointer<'a,std::os::raw::c_char>),
    VoidPointer(RawPointer<'a,std::os::raw::c_void>),
}

/*// ポインタとその保持者のセット*/
/*pub struct PointerWithHolder<T> {*/
/*pub pointer: RawPointer<T>,*/
/*pub holder: Option<Box<dyn std::any::Any>>, // 任意の所有権付きオブジェクト*/
/*}*/

// ポインター型の変換用トレイト
pub trait ToRawPointer<'a,T>: Sized {
    //fn to_raw_pointer(self) -> PointerWithHolder<T>;
    fn to_raw_pointer(self) -> RawPointer<'a,T>;
}
pub trait ToGeneralType<'a>:
    ToRawPointer<'a,std::os::raw::c_char> + ToRawPointer<'a,std::os::raw::c_void>
{
    fn to_general_type(self) -> GeneralType<'a>;
}
impl<'a> ToRawPointer<'a,std::os::raw::c_char> for String {
    //fn to_raw_pointer(self) -> PointerWithHolder<std::os::raw::c_char> {

    fn to_raw_pointer(self) -> RawPointer<'a,std::os::raw::c_char> {
        let cstring = CString::new(self).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(Box::new(cstring)))
    }
}
impl<'a> ToRawPointer<'a,std::os::raw::c_void> for String {
    //fn to_raw_pointer(self) -> PointerWithHolder<std::os::raw::c_void> {

    fn to_raw_pointer(self) -> RawPointer<'a,std::os::raw::c_void> {
        /*
        // CStringHolderに相当する処理
        let cstring = CString::new(self).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        std::mem::forget(cstring); // リークさせて明示的に保持する（要管理）
        RawPointer::Constant(ptr)
        */

        let ptr = RawPointer::new_const(std::ptr::null_mut(), None);
        ptr
    }
}

/*impl ToGeneralType for String {*/
/*fn to_general_type(self) -> GeneralType {*/
/*self.to_raw_pointer().into()*/
/*}*/
/*}*/

impl<'a> ToGeneralType<'a> for String {
    fn to_general_type(self) -> GeneralType<'a> {
        let raw_pointer = self.to_raw_pointer(); // これでポインタを取得*/
        match raw_pointer {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }
}

/*// RawPointer から GeneralType への変換*/
/*impl From<RawPointer<c_char>> for GeneralType {*/
/*fn from(ptr: RawPointer<c_char>) -> Self {*/
/*GeneralType::CharPointer(ptr)*/
/*}*/
/*}*/

/*// PointerWithHolder から GeneralType への変換*/
/*impl From<PointerWithHolder<c_char>> for GeneralType {*/
/*fn from(holder: PointerWithHolder<c_char>) -> Self {*/
/*match holder.pointer {*/
/*RawPointer::Constant(ptr) => GeneralType::CharPointer(RawPointer::Constant(ptr)),*/
/*RawPointer::Mutable(ptr) => GeneralType::CharPointer(RawPointer::Mutable(ptr)),*/

/*}*/
/*}*/
/*}*/

/*impl From<RawPointer<std::os::raw::c_char>> for GeneralType {*/
/*fn from(p: RawPointer<std::os::raw::c_char>) -> Self {*/
/*GeneralType::CharPointer(p, None)*/
/*}*/
/*}*/
