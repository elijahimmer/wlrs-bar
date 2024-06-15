use crate::draw::Rect;
use crate::utils::{max, min};

use num_traits::{FromPrimitive, Num, NumCast};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Point<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> {
    pub x: T,
    pub y: T,
}

impl<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    pub fn extend_to(self, x: T, y: T) -> Rect<T> {
        Rect::<T>::new(self, Self::new(x, y))
    }

    pub fn smallest(self, other: Self) -> Self {
        Self::new(min(self.x, other.x), min(self.y, other.y))
    }

    pub fn largest(self, other: Self) -> Self {
        Self::new(max(self.x, other.x), max(self.y, other.y))
    }
}

macro_rules! from_impl_same {
    ($($t:ident)*) => ($(
        impl From<rusttype::Point<$t>> for Point<$t> {
            fn from(val: rusttype::Point<$t>) -> Self {
                Self::new(val.x, val.y)
            }
        }

        impl From<Point<$t>> for rusttype::Point<$t> {
            fn from(val: Point<$t>) -> Self {
                Self { x: val.x, y: val.y }
            }
        }
            )*)
}

macro_rules! from_impl_other {
    ($t:ident, $f:ident) => {
        impl From<rusttype::Point<$t>> for Point<$f> {
            fn from(val: rusttype::Point<$t>) -> Self {
                Self::new(
                    <$f as NumCast>::from(val.x).unwrap(),
                    <$f as NumCast>::from(val.y).unwrap(),
                )
            }
        }

        impl From<Point<$t>> for rusttype::Point<$f> {
            fn from(val: Point<$t>) -> Self {
                Self {
                    x: <$f as NumCast>::from(val.x).unwrap(),
                    y: <$f as NumCast>::from(val.y).unwrap(),
                }
            }
        }

        impl From<($t, $t)> for Point<$f> {
            fn from(val: ($t, $t)) -> Self {
                Self {
                    x: <$f as NumCast>::from(val.0).unwrap(),
                    y: <$f as NumCast>::from(val.1).unwrap(),
                }
            }
        }
    };
}

macro_rules! from_impl_self {
    ($t:ident, $($f:ident)*) => ($(
        impl From<Point<$t>> for Point<$f> {
            fn from(val: Point<$t>) -> Self {
                Self::new(<$f as NumCast>::from(val.x).unwrap(), <$f as NumCast>::from(val.y).unwrap())
            }
        }

        from_impl_other!($t, $f);
        //from_impl_other!($t, $t);
    )*)
}

from_impl_self! { u8, u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { u16, u8 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { u32, u8 u16 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { u64, u8 u16 u32 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { u128, u8 u16 u32 u64 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { usize, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { i8, u8 u16 u32 u64 u128 usize i16 i32 i64 i128 isize f32 f64 }
from_impl_self! { i16, u8 u16 u32 u64 u128 usize i8 i32 i64 i128 isize f32 f64 }
from_impl_self! { i32, u8 u16 u32 u64 u128 usize i8 i16 i64 i128 isize f32 f64 }
from_impl_self! { i64, u8 u16 u32 u64 u128 usize i8 i16 i32 i128 isize f32 f64 }
from_impl_self! { i128, u8 u16 u32 u128 u64 usize i8 i16 i32 i64 isize f32 f64 }
from_impl_self! { isize, u8 u16 u32 usize u64 u128 i8 i16 i32 i64 i128 f32 f64 }
from_impl_self! { f32, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f64 }
from_impl_self! { f64, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 }
from_impl_same! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }

use std::ops::Add;
impl<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> Add<Self> for Point<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

use std::ops::Sub;
impl<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> Sub<Self> for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
