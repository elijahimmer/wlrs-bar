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

impl<T, U> From<(U, U)> for Point<T>
where
    T: FromPrimitive + NumCast + Num + Copy + PartialOrd,
    U: FromPrimitive + NumCast + Num + Copy + PartialOrd,
{
    fn from(v: (U, U)) -> Self {
        Self::new(
            T::from(v.0).expect("failed to convert number"),
            T::from(v.1).expect("failed to convert number"),
        )
    }
}

impl<T, U> From<rusttype::Point<U>> for Point<T>
where
    T: FromPrimitive + NumCast + Num + Copy + PartialOrd,
    U: FromPrimitive + NumCast + Num + Copy + PartialOrd,
{
    fn from(val: rusttype::Point<U>) -> Self {
        Self::new(
            T::from(val.x).expect("failed to convert number"),
            T::from(val.y).expect("failed to convert number"),
        )
    }
}

impl<T, U> From<Point<U>> for rusttype::Point<T>
where
    T: FromPrimitive + NumCast + Num + Copy + PartialOrd,
    U: FromPrimitive + NumCast + Num + Copy + PartialOrd,
{
    fn from(val: Point<U>) -> Self {
        Self {
            x: T::from(val.x).expect("failed to convert number"),
            y: T::from(val.y).expect("failed to convert number"),
        }
    }
}

impl From<Point<u32>> for Point<f32> {
    fn from(val: Point<u32>) -> Self {
        Self::new(val.x as f32, val.y as f32)
    }
}

impl From<Point<f32>> for Point<u32> {
    fn from(val: Point<f32>) -> Self {
        Self::new(val.x.ceil() as u32, val.y.ceil() as u32)
    }
}

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
