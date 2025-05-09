pub mod dxlib;
pub mod utils;

mod tests {
    use crate::dxlib::*;
    use anyhow::Result as R;
    use std::f64::consts::PI;
    use std::ffi::CStr;
    use std::os::raw::c_char;

    #[test]
    fn test_dxlib_1() -> R<(), DxLibError> {
        ChangeWindowMode(1)?;
        DxLib_Init()?;
        DxLib_End()?;
        Ok(())
    }
}
