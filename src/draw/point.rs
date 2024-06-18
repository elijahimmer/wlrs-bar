use crate::draw::Rect;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn extend_to(self, other: Self) -> Rect {
        Rect::new(self, other)
    }

    pub fn smallest(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn largest(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl From<rusttype::Point<u32>> for Point {
    fn from(val: rusttype::Point<u32>) -> Self {
        Self::new(val.x, val.y)
    }
}

impl From<Point> for rusttype::Point<u32> {
    fn from(val: Point) -> Self {
        Self { x: val.x, y: val.y }
    }
}

impl From<rusttype::Point<i32>> for Point {
    fn from(val: rusttype::Point<i32>) -> Self {
        Self::new(val.x as u32, val.y as u32)
    }
}

impl From<Point> for rusttype::Point<i32> {
    fn from(val: Point) -> Self {
        Self {
            x: val.x as i32,
            y: val.y as i32,
        }
    }
}

impl From<rusttype::Point<f32>> for Point {
    fn from(val: rusttype::Point<f32>) -> Self {
        Self::new(val.x.round() as u32, val.y.round() as u32)
    }
}

impl From<Point> for rusttype::Point<f32> {
    fn from(val: Point) -> Self {
        Self {
            x: val.x as f32,
            y: val.y as f32,
        }
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

use std::cmp::{Ordering, PartialOrd};
impl PartialOrd for Point {
    // Required method
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let x_cmp = self.x.cmp(&other.x);
        let y_cmp = self.y.cmp(&other.y);

        if x_cmp == y_cmp {
            Some(x_cmp)
        } else {
            None
        }
    }
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{} x {}", self.x, self.y)
    }
}
