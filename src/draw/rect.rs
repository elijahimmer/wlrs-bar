use crate::draw::{Align, DrawCtx, Point};
use crate::utils::cmp;
use num_traits::{FromPrimitive, Num, NumCast};

use wayland_client::protocol::wl_surface::WlSurface;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Rect<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> {
    pub min: Point<T>,
    pub max: Point<T>,
}

impl<T: FromPrimitive + NumCast + Num + Copy + PartialOrd> Rect<T> {
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
}

impl Rect<u32> {
    pub fn draw(self, color: crate::color::Color, ctx: &mut DrawCtx) {
        assert!(ctx.rect.contains_rect(self));

        for y in self.min.y..self.max.y {
            for x in self.min.x..self.max.x {
                ctx.put(Point { x, y }, color);
            }
        }
    }

    pub fn draw_outline(self, color: crate::color::Color, ctx: &mut DrawCtx) {
        assert!(ctx.rect.contains_rect(self));

        for x in self.min.x + 1..self.max.x {
            ctx.put(Point::new(x, self.min.y), color);
            ctx.put(Point::new(x, self.max.y - 1), color);
        }

        for y in self.min.y..self.max.y {
            ctx.put(Point::new(self.min.x, y), color);
            ctx.put(Point::new(self.max.x - 1, y), color);
        }
    }

    pub fn damage_outline(self, sur: WlSurface) {
        sur.damage(self.min.x as i32, self.min.y as i32, self.width() as i32, 1);
        sur.damage(self.min.x as i32, self.max.y as i32, self.width() as i32, 1);
        sur.damage(
            self.min.x as i32,
            self.min.y as i32,
            1,
            self.height() as i32,
        );
        sur.damage(
            self.max.x as i32,
            self.min.y as i32,
            1,
            self.height() as i32 + 1,
        );
    }
}

impl<T, U> From<rusttype::Rect<U>> for Rect<T>
where
    T: FromPrimitive + NumCast + Num + Copy + PartialOrd,
    U: FromPrimitive + NumCast + Num + Copy + PartialOrd,
{
    fn from(val: rusttype::Rect<U>) -> Self {
        Self::new(val.min.into(), val.max.into())
    }
}

impl From<Rect<u32>> for Rect<f32> {
    fn from(val: Rect<u32>) -> Self {
        Self::new(val.min.into(), val.max.into())
    }
}

impl From<Rect<f32>> for Rect<u32> {
    fn from(val: Rect<f32>) -> Self {
        Self::new(val.min.into(), val.max.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert() {
        let r1: Rect<f32> = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let r2: Rect<u32> = Rect::new(Point::new(0, 0), Point::new(1, 1));

        assert_eq!(r1, r2.into());
    }

    #[test]
    fn largest() {
        // normal
        let r1 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let r2 = Rect::new(Point::new(2.0, 2.0), Point::new(3.0, 3.0));

        let largest = r1.largest(r2);
        assert_eq!(largest.min, r1.min);
        assert_eq!(largest.max, r2.max);

        // bigger second
        let r1 = Rect::new(Point::new(2.0, 2.0), Point::new(3.0, 3.0));
        let r2 = Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));

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
