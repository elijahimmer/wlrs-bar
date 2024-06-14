use crate::draw::Rect;
use crate::utils::cmp;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }

    pub fn extend_to(self, x: u32, y: u32) -> Rect {
        let (min_x, max_x) = cmp(self.x, x);
        let (min_y, max_y) = cmp(self.y, y);

        Rect {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }

    pub fn min(self, other: Point) -> Point {
        let (min_x, _max_x) = cmp(self.x, other.x);
        let (min_y, _max_y) = cmp(self.y, other.y);

        Point { x: min_x, y: min_y }
    }

    pub fn max(self, other: Point) -> Point {
        let (_min_x, max_x) = cmp(self.x, other.x);
        let (_min_y, max_y) = cmp(self.y, other.y);

        Point { x: max_x, y: max_y }
    }
}

impl std::ops::Add<Self> for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl From<(u32, u32)> for Point {
    fn from(v: (u32, u32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<rusttype::Point<f32>> for Point {
    fn from(v: rusttype::Point<f32>) -> Self {
        Self {
            x: v.x as u32,
            y: v.y as u32,
        }
    }
}

impl From<rusttype::Point<i32>> for Point {
    fn from(v: rusttype::Point<i32>) -> Self {
        Self {
            x: v.x as u32,
            y: v.y as u32,
        }
    }
}

impl From<rusttype::Point<u32>> for Point {
    fn from(v: rusttype::Point<u32>) -> Self {
        Self { x: v.x, y: v.y }
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

impl From<Point> for rusttype::Point<u32> {
    fn from(val: Point) -> Self {
        Self { x: val.x, y: val.y }
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
