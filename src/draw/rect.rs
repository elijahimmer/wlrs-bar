use super::{Align, Color, DrawCtx, Point};
use crate::utils::cmp;

use wayland_client::protocol::wl_surface::WlSurface;

/**
 * A Rectangle to present area used on the screen.
 * The min should *always* be smaller than, not equal to,
 *  the max in both x and y.
 */
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(a: impl Into<Point>, b: impl Into<Point>) -> Self {
        let (a, b) = (a.into(), b.into());
        let (min_x, max_x) = cmp(a.x, b.x);
        let (min_y, max_y) = cmp(a.y, b.y);
        debug_assert!(min_x < max_x);
        debug_assert!(min_y < max_y);

        Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }

    pub fn width(self) -> u32 {
        debug_assert!(self.max > self.min);
        self.max.x - self.min.x
    }

    pub fn height(self) -> u32 {
        debug_assert!(self.max > self.min);
        self.max.y - self.min.y
    }

    pub fn size(self) -> Point {
        debug_assert!(self.max > self.min);
        Point::new(self.width(), self.height())
    }

    pub fn center(self) -> Point {
        debug_assert!(self.max > self.min);
        Point {
            x: (self.min.x + self.max.x) / 2,
            y: (self.min.y + self.max.y) / 2,
        }
    }

    pub fn largest(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        debug_assert!(self.max > self.min);
        Self {
            min: self.min.smallest(other.min),
            max: self.max.largest(other.max),
        }
    }

    pub fn smallest(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        debug_assert!(self.max > self.min);
        Self::new(self.min.largest(other.min), self.max.smallest(other.max))
    }

    pub fn x_shift(self, x_offset: i32) -> Self {
        debug_assert!(self.max > self.min);
        Self {
            min: self.min.x_shift(x_offset),
            max: self.max.x_shift(x_offset),
        }
    }

    pub fn y_shift(self, y_offset: i32) -> Self {
        debug_assert!(self.max > self.min);
        Self {
            min: self.min.y_shift(y_offset),
            max: self.max.y_shift(y_offset),
        }
    }

    pub fn place_at(self, size: Point, h_align: Align, v_align: Align) -> Self {
        //log::trace!("place_at :: self: {self}, size: {size}, {h_align:?} x {v_align:?}");
        debug_assert!(self.max > self.min);
        debug_assert!(self.max.x >= self.min.x + size.x);
        debug_assert!(self.max.y >= self.min.y + size.y);

        let align = |align, min, max, size| {
            let center = (min + max) / 2;
            let (min_res, max_res) = match align {
                Align::Start => (min, min + size),
                Align::End => (max - size, max),
                Align::Center => (center - (size / 2), center + (size / 2) + (size % 2)),
                Align::CenterAt(ratio) => {
                    debug_assert!((0.0..1.0).contains(&ratio));
                    let up = (size as f32 * (1.0 - ratio)).round() as u32;
                    if up > center || up > size || center - up < min {
                        (center - (size / 2), center + (size / 2) + (size % 2))
                    } else {
                        (center - up, center + (size - up) + (size % 2))
                    }
                }
            };

            //log::trace!(
            //    "place_at :: center: {center}, min_res: {min_res}, max_res: {max_res}, dist: {}",
            //    max_res - min_res,
            //);
            debug_assert!(min_res <= max_res);
            debug_assert!(max_res - min_res == size);

            (min_res, max_res)
        };

        let (x_min, x_max) = align(h_align, self.min.x, self.max.x, size.x);
        let (y_min, y_max) = align(v_align, self.min.y, self.max.y, size.y);

        let min = Point::new(x_min, y_min);
        let max = Point::new(x_max, y_max);

        debug_assert!(self.contains(min));
        debug_assert!(self.contains(max));

        //log::trace!("place_at :: result: {min} -> {max}");
        Self { min, max }
    }

    pub fn contains(self, p: impl Into<Point>) -> bool {
        let p = p.into();
        debug_assert!(self.max > self.min);
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    pub fn contains_rect(self, r: impl Into<Self>) -> bool {
        let r = r.into();
        debug_assert!(self.max > self.min);
        self.min.x <= r.min.x
            && self.min.y <= r.min.y
            && self.max.x >= r.max.x
            && self.max.y >= r.max.y
    }

    pub fn draw(self, color: Color, ctx: &mut DrawCtx) {
        debug_assert!(self.max > self.min);
        //log::debug!("draw :: self: {self}");
        for y in self.min.y..self.max.y {
            for x in self.min.x..self.max.x {
                ctx.put(Point { x, y }, color);
            }
        }
    }

    pub fn draw_outline(self, color: Color, ctx: &mut DrawCtx) {
        debug_assert!(self.max > self.min);
        for x in self.min.x + 1..self.max.x {
            ctx.put(Point::new(x, self.min.y), color);
            ctx.put(Point::new(x, self.max.y - 1), color);
        }

        for y in self.min.y..self.max.y {
            ctx.put(Point::new(self.min.x, y), color);
            ctx.put(Point::new(self.max.x - 1, y), color);
        }
    }

    pub fn damage_outline(self, surface: WlSurface) {
        debug_assert!(self.max > self.min);
        let x_min = self.min.x as i32;
        let x_max = self.max.x as i32 - 1;
        let y_min = self.min.y as i32;
        let y_max = self.max.y as i32 - 1;

        surface.damage(x_min, x_max, y_min, y_min);
        surface.damage(x_min, x_max, y_max, y_max);
        surface.damage(x_min, x_min, y_min, y_max);
        surface.damage(x_max, x_max, y_min, y_max);
    }
}

impl From<rusttype::Rect<i32>> for Rect {
    fn from(val: rusttype::Rect<i32>) -> Self {
        Self::new(val.min, val.max)
    }
}

use std::ops::Add;
impl Add<Self> for Rect {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

use std::ops::AddAssign;
impl AddAssign<Self> for Rect {
    fn add_assign(&mut self, rhs: Self) {
        self.min += rhs.min;
        self.max += rhs.max;
    }
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for Rect {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{} -> {}", self.min, self.max)
    }
}

#[cfg(test)]
mod test {}
