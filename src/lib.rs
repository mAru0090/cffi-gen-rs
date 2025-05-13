pub mod ffi {
    pub mod dxlib;
}
pub mod utils;

mod tests {
    use crate::ffi::dxlib::*;
    use anyhow::Result as R;
    use std::f64::consts::PI;
    use std::ffi::CStr;
    use std::os::raw::c_char;

    #[test]
    fn test_dxlib_1() -> R<(), DxLibError> {
        let mut string = String::from("test! hello world! おはよう!");
        SetUseCharCodeFormat(65001)?;
        ChangeWindowMode(1)?;
        DxLib_Init()?;
        let white_color = GetColor(255, 255, 255)?;
        DrawString(0, 0, &string, white_color)?;
        WaitKey()?;
        string += "あおいえｐ";
        DrawString(0, 0, &string, white_color)?;
        WaitKey()?;
        DxLib_End()?;
        Ok(())
    }
}
