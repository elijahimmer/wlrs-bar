use crate::draw::Rect;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn extend_to(self, other: impl Into<Self>) -> Rect {
        Rect::new(self, other.into())
    }

    pub fn smallest(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn largest(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn x_shift(self, offset: i32) -> Self {
        Self::new((self.x as i32 + offset).try_into().unwrap(), self.y)
    }

    pub fn y_shift(self, offset: i32) -> Self {
        Self::new(self.x, (self.y as i32 + offset).try_into().unwrap())
    }
}

macro_rules! for_each_primative {
    ($($t:ty)+) => ($(
        impl From<($t, $t)> for Point {
            fn from((x, y): ($t, $t)) -> Self {
                assert!(0 as $t <= x, "x: {x}");
                assert!(0 as $t <= y, "y: {y}");
                Self::new(x as u32, y.clamp(0 as $t, (u32::MAX) as $t) as u32)
            }
        }

        impl From<rusttype::Point<$t>> for Point {
            fn from(val: rusttype::Point<$t>) -> Self {
                assert!(0 as $t <= val.x, "x: {}", val.x);
                assert!(0 as $t <= val.y, "y: {}", val.y);
                Self::new(val.x as u32, val.y as u32)
            }
        }

        impl From<Point> for rusttype::Point<$t> {
            fn from(val: Point) -> Self {
                Self { x: val.x as $t, y: val.y as $t }
            }
        }
    )*)
}

for_each_primative!(u8 u16 u32 u64 i8 i16 i32 i64 f32 f64);

use std::ops::Add;
impl Add<Self> for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

use std::ops::AddAssign;
impl AddAssign<Self> for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

use std::ops::Sub;
impl Sub<Self> for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

use std::ops::Mul;
impl Mul<u32> for Point {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

use std::ops::Div;
impl Div<u32> for Point {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "({} x {})", self.x, self.y)
    }
}
