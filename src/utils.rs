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
pub enum Owner {
    Int(std::os::raw::c_int),
    Float(std::os::raw::c_float),
    CString(CString),
}
pub enum OwnerSlice {
    Int(Box<[std::os::raw::c_int]>),
    Float(Box<[std::os::raw::c_float]>),
    Char(Box<[std::os::raw::c_char]>),
}
pub enum RawOwner {
    Plain(Owner),
    Slice(OwnerSlice),
}
pub enum RawPointer<T> {
    Mutable {
        ptr: *mut T,
        owner: Option<RawOwner>,
        _marker: PhantomData<T>,
    },
    Constant {
        ptr: *const T,
        owner: Option<RawOwner>,
        _marker: PhantomData<T>,
    },
}
impl<T> RawPointer<T> {
    pub fn new_mut(ptr: *mut T, owner: Option<RawOwner>) -> Self {
        RawPointer::Mutable {
            ptr,
            owner,
            _marker: PhantomData,
        }
    }

    pub fn new_const(ptr: *const T, owner: Option<RawOwner>) -> Self {
        RawPointer::Constant {
            ptr,
            owner,
            _marker: PhantomData,
        }
    }
}
pub enum GeneralRawType {
    IntPointer(RawPointer<std::os::raw::c_int>),
    FloatPointer(RawPointer<std::os::raw::c_float>),
    DoublePointer(RawPointer<std::os::raw::c_double>),
    CharPointer(RawPointer<std::os::raw::c_char>),
    VoidPointer(RawPointer<std::os::raw::c_void>),
}

// ポインター型の変換用トレイト
pub trait ToRawPointer<T>: Sized {
    fn to_raw_pointer_owned(self) -> RawPointer<T>;
    fn to_raw_pointer_ref(&self) -> RawPointer<T>;
    fn to_raw_pointer_mut(&mut self) -> RawPointer<T>;
}

pub trait ToGeneralRawType {
    //fn to_general_type(&'a self) -> GeneralRawType<'a>;

    fn to_general_type_owned(self) -> GeneralRawType;
    fn to_general_type_ref(&self) -> GeneralRawType;
    fn to_general_type_mut(&mut self) -> GeneralRawType;
}

// &str → c_char
impl ToRawPointer<std::os::raw::c_char> for &str {
    fn to_raw_pointer_owned(self) -> RawPointer<std::os::raw::c_char> {
        let cstring = CString::new(self).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Plain(Owner::CString(cstring))))
    }

    fn to_raw_pointer_ref(&self) -> RawPointer<std::os::raw::c_char> {
        let cstring = CString::new(*self).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Plain(Owner::CString(cstring))))
    }

    fn to_raw_pointer_mut(&mut self) -> RawPointer<std::os::raw::c_char> {
        let cstring = CString::new(*self).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Plain(Owner::CString(cstring))))
    }
}
// String → c_char
impl ToRawPointer<std::os::raw::c_char> for String {
    fn to_raw_pointer_owned(self) -> RawPointer<std::os::raw::c_char> {
        let cstring = CString::new(self).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Plain(Owner::CString(cstring))))
    }

    fn to_raw_pointer_ref(&self) -> RawPointer<std::os::raw::c_char> {
        let cstring = CString::new(self.clone()).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Plain(Owner::CString(cstring))))
    }

    fn to_raw_pointer_mut(&mut self) -> RawPointer<std::os::raw::c_char> {
        let cstring = CString::new(self.clone()).expect("CString conversion failed");
        let ptr = cstring.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Plain(Owner::CString(cstring))))
    }
}

impl ToRawPointer<std::os::raw::c_char> for &[std::os::raw::c_char] {
    fn to_raw_pointer_owned(self) -> RawPointer<std::os::raw::c_char> {
        // 共有参照はownedにはなりえないので、cloneしてBoxにする
        let boxed: Box<[c_char]> = self.to_vec().into_boxed_slice();
        let ptr = boxed.as_ptr();
        RawPointer::new_const(ptr, Some(RawOwner::Slice(OwnerSlice::Char(boxed))))
    }

    fn to_raw_pointer_ref(&self) -> RawPointer<std::os::raw::c_char> {
        let ptr = self.as_ptr();
        RawPointer::new_const(ptr, None)
    }

    fn to_raw_pointer_mut(&mut self) -> RawPointer<std::os::raw::c_char> {
        // 共有参照からはmutは無理なのでpanicしてもいい
        panic!("Cannot get mutable pointer from immutable slice reference");
    }
}

impl ToGeneralRawType for &[std::os::raw::c_char] {
    fn to_general_type_owned(self) -> GeneralRawType {
        match self.to_raw_pointer_owned() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_ref(&self) -> GeneralRawType {
        match self.to_raw_pointer_ref() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_mut(&mut self) -> GeneralRawType {
        match self.to_raw_pointer_mut() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }
}

impl ToRawPointer<std::os::raw::c_char> for &mut Vec<c_char> {
    fn to_raw_pointer_owned(self) -> RawPointer<std::os::raw::c_char> {
        // Vec自体の所有権がないので、cloneしてBoxにする
        let mut boxed: Box<[std::os::raw::c_char]> = self.clone().into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        RawPointer::new_mut(ptr, Some(RawOwner::Slice(OwnerSlice::Char(boxed))))
    }

    fn to_raw_pointer_ref(&self) -> RawPointer<std::os::raw::c_char> {
        let ptr = self.as_ptr();
        RawPointer::new_const(ptr, None)
    }

    fn to_raw_pointer_mut(&mut self) -> RawPointer<std::os::raw::c_char> {
        let ptr = self.as_mut_ptr();
        RawPointer::new_mut(ptr, None)
    }
}

impl ToGeneralRawType for &mut Vec<std::os::raw::c_char> {
    fn to_general_type_owned(self) -> GeneralRawType {
        match self.to_raw_pointer_owned() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_ref(&self) -> GeneralRawType {
        match self.to_raw_pointer_ref() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_mut(&mut self) -> GeneralRawType {
        match self.to_raw_pointer_mut() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }
}

impl ToRawPointer<std::os::raw::c_char> for &mut [std::os::raw::c_char] {
    fn to_raw_pointer_owned(self) -> RawPointer<std::os::raw::c_char> {
        // 所有権がすでにあるため、そのまま Box にできる
        let mut boxed: Box<[std::os::raw::c_char]> = self.to_vec().into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        RawPointer::new_mut(ptr, Some(RawOwner::Slice(OwnerSlice::Char(boxed))))
    }

    fn to_raw_pointer_ref(&self) -> RawPointer<std::os::raw::c_char> {
        let ptr = self.as_ptr();
        RawPointer::new_const(ptr, None)
    }

    fn to_raw_pointer_mut(&mut self) -> RawPointer<std::os::raw::c_char> {
        let ptr = self.as_mut_ptr();
        RawPointer::new_mut(ptr, None)
    }
}

impl ToGeneralRawType for &mut [std::os::raw::c_char] {
    fn to_general_type_owned(self) -> GeneralRawType {
        match self.to_raw_pointer_owned() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_ref(&self) -> GeneralRawType {
        match self.to_raw_pointer_ref() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_mut(&mut self) -> GeneralRawType {
        match self.to_raw_pointer_mut() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }
}

impl ToGeneralRawType for String {
    fn to_general_type_owned(self) -> GeneralRawType {
        match self.to_raw_pointer_owned() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_ref(&self) -> GeneralRawType {
        match self.to_raw_pointer_ref() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_mut(&mut self) -> GeneralRawType {
        match self.to_raw_pointer_mut() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }
}

impl ToGeneralRawType for &str {
    fn to_general_type_owned(self) -> GeneralRawType {
        match self.to_raw_pointer_owned() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_ref(&self) -> GeneralRawType {
        match self.to_raw_pointer_ref() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }

    fn to_general_type_mut(&mut self) -> GeneralRawType {
        match self.to_raw_pointer_mut() {
            RawPointer::Constant { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_const(ptr, owner))
            }
            RawPointer::Mutable { ptr, owner, .. } => {
                GeneralRawType::CharPointer(RawPointer::new_mut(ptr, owner))
            }
        }
    }
}
