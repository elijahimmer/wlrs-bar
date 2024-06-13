use lazy_static::lazy_static;
use rusttype::Font as RustFont;

pub static DEJAVUSANS_MONO_FONT_DATA: &[u8] = include_bytes!("../fonts/dejavu/DejaVuSansMono.ttf");
pub static ROBOTO_REGULAR_FONT_DATA: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");

lazy_static! {
    pub static ref DEJAVUSANS_MONO: RustFont<'static> =
        RustFont::try_from_bytes(DEJAVUSANS_MONO_FONT_DATA as &[u8])
            .expect("error constructing DejaVuSansMono");
    pub static ref ROBOTO_REGULAR: RustFont<'static> =
        RustFont::try_from_bytes(ROBOTO_REGULAR_FONT_DATA as &[u8])
            .expect("error constructing Roboto-Regular");
}
