use lazy_static::lazy_static;
use rusttype::{Font, Scale};

pub static FONT_DATA: &[u8] = include_bytes!("../fonts/FiraCodeNerdFontMono-Regular.ttf");

lazy_static! {
    pub static ref FONT: Font<'static> =
        Font::try_from_bytes(FONT_DATA as &[u8]).expect("error constructing FiraCodeMono");
}

pub fn font_width(scale: Scale) -> f32 {
    FONT.glyph('0').scaled(scale).h_metrics().advance_width
}
