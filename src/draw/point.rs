use crate::draw::Rect;
use num_traits::{AsPrimitive, FromPrimitive};

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

impl<T: AsPrimitive<u32>> From<(T, T)> for Point {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x.as_(), y.as_())
    }
}

impl<T: AsPrimitive<u32>> From<rusttype::Point<T>> for Point {
    fn from(val: rusttype::Point<T>) -> Self {
        Self::new(val.x.as_(), val.y.as_())
    }
}

impl<T: FromPrimitive> From<Point> for rusttype::Point<T> {
    fn from(val: Point) -> Self {
        let x = T::from_u32(val.x)
            .unwrap_or_else(|| panic!("X cannot fit in {}. val: {val}", stringify!(T)));
        let y = T::from_u32(val.y)
            .unwrap_or_else(|| panic!("Y cannot fit in {}. val: {val}", stringify!(T)));
        Self { x, y }
    }
}

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
