use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn new_int(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r as f32 / 255.0).clamp(0.0, 1.0),
            g: (g as f32 / 255.0).clamp(0.0, 1.0),
            b: (b as f32 / 255.0).clamp(0.0, 1.0),
            a: (a as f32 / 255.0).clamp(0.0, 1.0),
        }
    }

    pub fn dilute(self, strength: f32) -> Self {
        Self {
            r: self.r,
            b: self.b,
            g: self.g,
            a: strength.clamp(0.0, 1.0),
        }
    }

    pub fn blend(self, other: Self, mut ratio: f32) -> Self {
        ratio = ratio.clamp(0.0, 1.0);
        Self {
            r: self.r + ((other.r - self.r) * ratio),
            g: self.g + ((other.g - self.g) * ratio),
            b: self.b + ((other.b - self.b) * ratio),
            a: self.a + ((other.a - self.a) * ratio),
        }
    }

    pub fn argb8888(self) -> [u8; 4] {
        let a = (self.a * 255.0) as u32;
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        ((a << 24) + (r << 16) + (g << 8) + b).to_le_bytes()
    }
}

lazy_static! {
    pub static ref BASE: Color = Color::new_int(0x19, 0x17, 0x24, 0xFF);
    pub static ref SURFACE: Color = Color::new_int(0x1f, 0x1d, 0x2e, 0xFF);
    pub static ref OVERLAY: Color = Color::new_int(0x26, 0x23, 0x3a, 0xFF);
    pub static ref MUTED: Color = Color::new_int(0x6e, 0x6a, 0x86, 0xFF);
    pub static ref SUBTLE: Color = Color::new_int(0x90, 0x8c, 0xaa, 0xFF);
    pub static ref TEXT: Color = Color::new_int(0xe0, 0xde, 0xf4, 0xFF);
    pub static ref LOVE: Color = Color::new_int(0xeb, 0x6f, 0x92, 0xFF);
    pub static ref GOLD: Color = Color::new_int(0xf6, 0xc1, 0x77, 0xFF);
    pub static ref ROSE: Color = Color::new_int(0xeb, 0xbc, 0xba, 0xFF);
    pub static ref PINE: Color = Color::new_int(0x31, 0x74, 0x8f, 0xFF);
    pub static ref FOAM: Color = Color::new_int(0x9c, 0xcf, 0xd8, 0xFF);
    pub static ref IRIS: Color = Color::new_int(0xc4, 0xa7, 0xe7, 0xFF);
    pub static ref H_LOW: Color = Color::new_int(0x21, 0x20, 0x2e, 0xFF);
    pub static ref H_MED: Color = Color::new_int(0x40, 0x3d, 0x52, 0xFF);
    pub static ref H_HIGH: Color = Color::new_int(0x52, 0x4f, 0x67, 0xFF);
}
