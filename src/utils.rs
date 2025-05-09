use std::ffi::CString;
use std::os::raw::c_char;
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

// ポインター型のラップ
pub enum RawPointer<T> {
    Mutable(*mut T),
    Constant(*const T),
}
// ポインター型の変換用トレイト
trait ToRawPointer<T>: Sized {
    fn to_raw_pointer(self) -> RawPointer<T>;
}

// &Stringでの定数ポインタ変換
impl<'a> ToRawPointer<u8> for &'a String {
    fn to_raw_pointer(self) -> RawPointer<u8> {
        RawPointer::Constant(self.as_ptr())
    }
}

// &mut String での可変ポインタ変換
impl<'a> ToRawPointer<u8> for &'a mut String {
    fn to_raw_pointer(self) -> RawPointer<u8> {
        RawPointer::Mutable(self.as_mut_ptr())
    }
}
