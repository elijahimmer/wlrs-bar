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

    /// Blend a base color (self) with a new color (other)
    pub fn blend(self, other: Self, ratio: f32) -> Self {
        assert!((-0.1..=1.1).contains(&ratio));
        let ratio = ratio.clamp(0.0, 1.0);
        Self {
            r: self.r + ((other.r as f32 - self.r as f32) * ratio) as u8,
            g: self.g + ((other.g as f32 - self.g as f32) * ratio) as u8,
            b: self.b + ((other.b as f32 - self.b as f32) * ratio) as u8,
            a: self.a + ((other.a as f32 - self.a as f32) * ratio) as u8,
        }
    }

    /// Returns a solid color by compositing a (possibly) transparent color (self)
    ///     onto the base color (onto)
    pub fn composite(self, onto: Self) -> Self {
        let ratio = self.a as f32 / 255.0;
        let ratio_old = 1.0 - ratio;
        let (r_new, g_new, b_new) = (self.r as f32, self.g as f32, self.b as f32);
        let (r_old, g_old, b_old) = (onto.r as f32, onto.g as f32, onto.b as f32);

        Self {
            r: (ratio * r_new + ratio_old * r_old).clamp(0.0, 255.0) as u8,
            g: (ratio * g_new + ratio_old * g_old).clamp(0.0, 255.0) as u8,
            b: (ratio * b_new + ratio_old * b_old).clamp(0.0, 255.0) as u8,
            a: self.a.saturating_add(onto.a),
        }
    }

    /// set the alpha (opacity) of the color
    pub fn dilute(self, alpha: u8) -> Self {
        Self { a: alpha, ..self }
    }

    /// set the alpha to the ratio provided.
    pub fn dilute_f32(self, alpha: f32) -> Self {
        assert!((-0.1..=1.1).contains(&alpha));
        let alpha = alpha.clamp(0.0, 1.0);
        Self {
            a: (alpha * 255.0) as u8,
            ..self
        }
    }

    /// Converts the color into argb8888
    pub fn argb8888(self) -> [u8; 4] {
        let a = (self.a as u32) << 24;
        let r = (self.r as u32) << 16;
        let g = (self.g as u32) << 8;
        let b = self.b as u32;
        (a + r + g + b).to_le_bytes()
    }

    /// Creates a color from a argb8888 array
    pub fn from_argb8888(argb: &[u8; 4]) -> Self {
        let color = u32::from_le_bytes(*argb);
        Self {
            a: (color >> 24) as u8,
            r: (color >> 16) as u8,
            g: (color >> 8) as u8,
            b: color as u8,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        FOAM // the default so you can easily see it's not set :)
    }
}

/// Macro to display color names instead of their hex values
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
        display_name!(f, *self, CLEAR BASE SURFACE OVERLAY MUTED SUBTLE TEXT LOVE GOLD ROSE PINE FOAM IRIS H_LOW H_MED H_HIGH);

        write!(f, "({:x} {:x} {:x} {:x})", self.r, self.g, self.b, self.a)
    }
}

pub const CLEAR: Color = Color::new(0, 0, 0, 0);
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

#[cfg(test)]
mod tests {
    use super::*;

    pub const ALL_COLORS: [Color; 16] = [
        CLEAR, BASE, SURFACE, OVERLAY, MUTED, SUBTLE, TEXT, LOVE, GOLD, ROSE, PINE, FOAM, IRIS, H_LOW,
        H_MED, H_HIGH,
    ];

    #[test]
    fn composite() {
        for color in ALL_COLORS {
            assert_eq!(CLEAR.composite(color), color);
        }

        for bg in ALL_COLORS {
            for fg in ALL_COLORS {
                if fg.a == u8::MAX {
                    assert_eq!(fg.composite(bg), fg);
                }
            }
            assert_eq!(bg.composite(CLEAR), bg);
        }
    }
}
