use super::{Align, DrawCtx, Point};
use crate::utils::cmp;

//use wayland_client::protocol::wl_surface::WlSurface;

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
    pub fn new(a: Point, b: Point) -> Self {
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

    pub fn largest(self, other: Self) -> Self {
        debug_assert!(self.max > self.min);
        Self {
            min: self.min.smallest(other.min),
            max: self.max.largest(other.max),
        }
    }

    pub fn smallest(self, other: Self) -> Self {
        debug_assert!(self.max > self.min);
        Self::new(self.min.largest(other.min), self.max.smallest(other.max))
    }

    pub fn place_at(self, size: Point, h_align: Align, v_align: Align) -> Self {
        log::trace!("place_at :: self: {self}, size: {size}, {h_align:?} x {v_align:?}");
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

            log::trace!(
                "place_at :: center: {center}, min_res: {min_res}, max_res: {max_res}, dist: {}",
                max_res - min_res,
                //center - min_res,
            );
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

        log::trace!("place_at :: result: {min} -> {max}");
        Self { min, max }
    }

    pub fn contains(self, p: Point) -> bool {
        debug_assert!(self.max > self.min);
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    pub fn contains_rect(self, r: Self) -> bool {
        debug_assert!(self.max > self.min);
        self.min.x <= r.min.x
            && self.min.y <= r.min.y
            && self.max.x >= r.max.x
            && self.max.y >= r.max.y
    }

    pub fn draw(self, color: crate::color::Color, ctx: &mut DrawCtx) {
        debug_assert!(self.max > self.min);
        log::debug!("draw :: self: {self}");
        for y in self.min.y..self.max.y {
            for x in self.min.x..self.max.x {
                ctx.put(Point { x, y }, color);
            }
        }
    }

    pub fn draw_outline(self, color: crate::color::Color, ctx: &mut DrawCtx) {
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
}

impl From<rusttype::Rect<i32>> for Rect {
    fn from(val: rusttype::Rect<i32>) -> Self {
        Self::new(val.min.into(), val.max.into())
    }
}

impl From<Rect> for rusttype::Rect<i32> {
    fn from(val: Rect) -> Self {
        Self {
            min: val.min.into(),
            max: val.max.into(),
        }
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
