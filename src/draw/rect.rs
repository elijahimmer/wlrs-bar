use crate::draw::{Align, DrawCtx, Point};
use crate::utils::cmp;
use num_traits::{FromPrimitive, Num, NumCast};

use wayland_client::protocol::wl_surface::WlSurface;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Rect<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> {
    pub min: Point<T>,
    pub max: Point<T>,
}

impl<T> Rect<T>
where
    T: FromPrimitive + NumCast + Num + Copy + PartialOrd,
{
    pub fn new(a: Point<T>, b: Point<T>) -> Self {
        let (min_x, max_x) = cmp(a.x, b.x);
        let (min_y, max_y) = cmp(a.y, b.y);
        debug_assert!(min_x <= max_x);
        debug_assert!(min_y <= max_y);

        Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }

    pub fn place_at(self, size: Point<T>, h_align: Align, v_align: Align) -> Self {
        assert!(self.max.x >= self.min.x + size.x);
        assert!(self.max.y >= self.min.y + size.y);

        let center_x = (self.min.x + self.max.x) / T::from_u32(2).unwrap();
        let (min_x, max_x) = match h_align {
            Align::Start => (self.min.x, self.min.x + size.x),
            Align::End => (self.max.x - size.x, self.max.x),
            Align::Center => (
                center_x - size.x / T::from_u32(2).unwrap(),
                center_x + size.x / T::from_u32(2).unwrap(),
            ),
        };
        assert!(min_x <= max_x);

        let center_y = (self.min.y + self.max.y) / T::from_u32(2).unwrap();
        let (min_y, max_y) = match v_align {
            Align::Start => (self.min.y, self.min.y + size.y),
            Align::End => (self.max.y - size.y, self.max.y),
            Align::Center => (
                center_y - size.y / T::from_u32(2).unwrap(),
                center_y + size.y / T::from_u32(2).unwrap(),
            ),
        };
        debug_assert!(min_y <= max_y);

        let min = Point::new(min_x, min_y);
        let max = Point::new(max_x, max_y);

        debug_assert!(self.contains(min));
        debug_assert!(self.contains(max));

        Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }

    pub fn width(self) -> T {
        self.max.x - self.min.x
    }

    pub fn height(self) -> T {
        self.max.y - self.min.y
    }

    pub fn size(self) -> Point<T> {
        Point::new(self.width(), self.height())
    }

    pub fn center(self) -> Point<T> {
        Point {
            x: (self.min.x + self.max.x) / T::from_u32(2).unwrap(),
            y: (self.min.y + self.max.y) / T::from_u32(2).unwrap(),
        }
    }

    pub fn largest(self, other: Rect<T>) -> Rect<T> {
        Self {
            min: self.min.smallest(other.min),
            max: self.max.largest(other.max),
        }
    }

    pub fn smallest(self, other: Rect<T>) -> Rect<T> {
        Self::new(self.min.largest(other.min), self.max.smallest(other.max))
    }

    pub fn contains(self, p: Point<T>) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    pub fn contains_rect(self, r: Self) -> bool {
        self.min.x <= r.min.x
            && self.min.y <= r.min.y
            && self.max.x >= r.max.x
            && self.max.y >= r.max.y
    }

    pub fn draw(self, color: crate::color::Color, ctx: &mut DrawCtx) {
        //assert!(ctx.rect.contains_rect(self.into()));

        let y_min: u32 = <u32 as NumCast>::from(self.min.y).unwrap();
        let y_max: u32 = <u32 as NumCast>::from(self.max.y).unwrap();
        let x_min: u32 = <u32 as NumCast>::from(self.min.x).unwrap();
        let x_max: u32 = <u32 as NumCast>::from(self.max.x).unwrap();

        for y in y_min..y_max {
            for x in x_min..x_max {
                ctx.put(Point { x, y }, color);
            }
        }
    }

    pub fn draw_outline(self, color: crate::color::Color, ctx: &mut DrawCtx) {
        //assert!(ctx.rect.contains_rect(self.into()));

        let x_min: u32 = <u32 as NumCast>::from(self.min.x).unwrap();
        let x_max: u32 = <u32 as NumCast>::from(self.max.x).unwrap();
        let y_min: u32 = <u32 as NumCast>::from(self.min.y).unwrap();
        let y_max: u32 = <u32 as NumCast>::from(self.max.y).unwrap() - 1;

        for x in x_min + 1..x_max {
            ctx.put(Point::new(x, y_min), color);
            ctx.put(Point::new(x, y_max), color);
        }

        for y in y_min..y_max {
            ctx.put(Point::new(x_min, y), color);
            ctx.put(Point::new(x_max - 1, y), color);
        }
    }

    pub fn damage_outline(self, sur: WlSurface) {
        let x_min: i32 = <i32 as NumCast>::from(self.min.x).unwrap();
        let x_max: i32 = <i32 as NumCast>::from(self.max.x).unwrap();
        let y_min: i32 = <i32 as NumCast>::from(self.min.y).unwrap();
        let y_max: i32 = <i32 as NumCast>::from(self.max.y).unwrap();
        let width: i32 = <i32 as NumCast>::from(self.width()).unwrap();
        let height: i32 = <i32 as NumCast>::from(self.height()).unwrap();

        sur.damage(x_min, y_min, width, 1);
        sur.damage(x_min, y_max, width, 1);
        sur.damage(x_min, y_min, 1, height);
        sur.damage(x_max, y_min, 1, height);
    }
}

macro_rules! from_impl_other {
    ($t:ident, $f:ident) => {
        impl From<rusttype::Rect<$t>> for Rect<$f> {
            fn from(val: rusttype::Rect<$t>) -> Self {
                Self::new(val.min.into(), val.max.into())
            }
        }

        impl From<Rect<$t>> for rusttype::Rect<$f> {
            fn from(val: Rect<$t>) -> Self {
                Self {
                    min: val.min.into(),
                    max: val.max.into(),
                }
            }
        }

        impl From<(($t, $t), ($t, $t))> for Rect<$f> {
            fn from(val: (($t, $t), ($t, $t))) -> Self {
                Self {
                    min: val.0.into(),
                    max: val.1.into(),
                }
            }
        }

        impl From<(Point<$t>, Point<$t>)> for Rect<$f> {
            fn from(val: (Point<$t>, Point<$t>)) -> Self {
                Self {
                    min: val.0.into(),
                    max: val.1.into(),
                }
            }
        }
    };
}

macro_rules! from_impl {
    ($t:ident, $($f:ident)*) => ($(
        impl From<Rect<$t>> for Rect<$f> {
            fn from(val: Rect<$t>) -> Self {
                Self::new(val.min.into(), val.max.into())
            }
        }

        from_impl_other!($t, $f);
    )*)
}

from_impl! { u8, u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl! { u16, u8 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl! { u32, u8 u16 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl! { u64, u8 u16 u32 u128 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl! { u128, u8 u16 u32 u64 usize i8 i16 i32 i64 i128 isize f32 f64 }
from_impl! { usize, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 isize f32 f64 }
from_impl! { i8, u8 u16 u32 u64 u128 usize i16 i32 i64 i128 isize f32 f64 }
from_impl! { i16, u8 u16 u32 u64 u128 usize i8 i32 i64 i128 isize f32 f64 }
from_impl! { i32, u8 u16 u32 u64 u128 usize i8 i16 i64 i128 isize f32 f64 }
from_impl! { i64, u8 u16 u32 u64 u128 usize i8 i16 i32 i128 isize f32 f64 }
from_impl! { i128, u8 u16 u32 u128 u64 usize i8 i16 i32 i64 isize f32 f64 }
from_impl! { isize, u8 u16 u32 usize u64 u128 i8 i16 i32 i64 i128 f32 f64 }
from_impl! { f32, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f64 }
from_impl! { f64, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert() {
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let r2 = Rect::new(Point::new(0, 0), Point::new(1, 1));

        assert_eq!(r1, r2.into());
    }

    #[test]
    fn largest() {
        // normal
        let r1 = Rect::new(Point::new(0, 0), Point::new(1, 1));
        let r2 = Rect::new(Point::new(2, 2), Point::new(3, 3));

        let largest = r1.largest(r2);
        assert_eq!(largest.min, r1.min);
        assert_eq!(largest.max, r2.max);

        // bigger second
        let r1 = Rect::new(Point::new(2, 2), Point::new(3, 3));
        let r2 = Rect::new(Point::new(0, 0), Point::new(1, 1));

        let largest = r1.largest(r2);
        assert_eq!(largest.min, r2.min);
        assert_eq!(largest.max, r1.max);

        // side by side
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let r2 = Rect::new(Point::new(0.2, 0.0), Point::new(3.0, 1.0));

        let val = r1.largest(r2);
        assert_eq!(val.min, Point::new(0.0, 0.0));
        assert_eq!(val.max, Point::new(3.0, 1.0));

        // inside another
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(3.0, 3.0));
        let r2 = Rect::new(Point::new(1.0, 1.0), Point::new(2.0, 2.0));

        assert_eq!(r1.largest(r2), r1);
    }

    #[test]
    fn smallest() {
        // normal
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let r2 = Rect::new(Point::new(2.0, 2.0), Point::new(3.0, 3.0));

        let val = r1.smallest(r2);
        assert_eq!(val.min, r1.max);
        assert_eq!(val.max, r2.min);

        // reverse order
        let r1 = Rect::new(Point::new(2.0, 2.0), Point::new(3.0, 3.0));
        let r2 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));

        let val = r1.smallest(r2);
        assert_eq!(val.min, r2.max);
        assert_eq!(val.max, r1.min);

        // side by side
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let r2 = Rect::new(Point::new(2.0, 0.0), Point::new(3.0, 1.0));

        let val = r1.smallest(r2);
        assert_eq!(val.min, Point::new(1.0, 0.0));
        assert_eq!(val.max, Point::new(2.0, 1.0));

        // inside another
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(3.0, 3.0));
        let r2 = Rect::new(Point::new(1.0, 1.0), Point::new(2.0, 2.0));

        assert_eq!(r1.smallest(r2), r2);
    }
}
