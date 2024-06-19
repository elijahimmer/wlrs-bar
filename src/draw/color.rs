#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn blend(self, other: Self, mut ratio: f32) -> Self {
        ratio = ratio.clamp(0.0, 1.0);
        Self {
            r: self.r + ((other.r as f32 - self.r as f32) * ratio) as u8,
            g: self.g + ((other.g as f32 - self.g as f32) * ratio) as u8,
            b: self.b + ((other.b as f32 - self.b as f32) * ratio) as u8,
            a: self.a + ((other.a as f32 - self.a as f32) * ratio) as u8,
        }
    }

    pub fn argb8888(self) -> [u8; 4] {
        let a = (self.a as u32) << 24;
        let r = (self.r as u32) << 16;
        let g = (self.g as u32) << 8;
        let b = self.b as u32;
        (a + r + g + b).to_le_bytes()
    }
}

impl Default for Color {
    fn default() -> Self {
        FOAM // the default so you can easily see it's not set :)
    }
}

macro_rules! display_name {
    ($fmt:ident, $self:expr, $($other:ident)+) => {
        $(if ($self == $other) {
            return write!($fmt, stringify!($other));
        })*
    }
}

use std::fmt::{Display, Error as DisplayError, Formatter};
impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), DisplayError> {
        display_name!(f, *self, BASE SURFACE OVERLAY MUTED SUBTLE TEXT LOVE GOLD ROSE PINE FOAM IRIS H_LOW H_MED H_HIGH);

        write!(f, "({:x} {:x} {:x} {:x})", self.r, self.g, self.b, self.a)
    }
}

pub const BASE: Color = Color::new(0x19, 0x17, 0x24, 0xFF);
pub const SURFACE: Color = Color::new(0x1f, 0x1d, 0x2e, 0xFF);
pub const OVERLAY: Color = Color::new(0x26, 0x23, 0x3a, 0xFF);
pub const MUTED: Color = Color::new(0x6e, 0x6a, 0x86, 0xFF);
pub const SUBTLE: Color = Color::new(0x90, 0x8c, 0xaa, 0xFF);
pub const TEXT: Color = Color::new(0xe0, 0xde, 0xf4, 0xFF);
pub const LOVE: Color = Color::new(0xeb, 0x6f, 0x92, 0xFF);
pub const GOLD: Color = Color::new(0xf6, 0xc1, 0x77, 0xFF);
pub const ROSE: Color = Color::new(0xeb, 0xbc, 0xba, 0xFF);
pub const PINE: Color = Color::new(0x31, 0x74, 0x8f, 0xFF);
pub const FOAM: Color = Color::new(0x9c, 0xcf, 0xd8, 0xFF);
pub const IRIS: Color = Color::new(0xc4, 0xa7, 0xe7, 0xFF);
pub const H_LOW: Color = Color::new(0x21, 0x20, 0x2e, 0xFF);
pub const H_MED: Color = Color::new(0x40, 0x3d, 0x52, 0xFF);
pub const H_HIGH: Color = Color::new(0x52, 0x4f, 0x67, 0xFF);
