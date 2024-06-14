use crate::draw::{DrawCtx, Point};
use crate::utils::cmp;

use wayland_client::protocol::wl_surface::WlSurface;

// which edge to align to
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(a: Point, b: Point) -> Self {
        let (min_x, max_x) = cmp(a.x, b.x);
        let (min_y, max_y) = cmp(a.y, b.y);

        Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }

    pub fn place_at(container: Rect, size: Point, h_align: Align, v_align: Align) -> Self {
        assert!(container.max.x >= container.min.x + size.x);
        assert!(container.max.y >= container.min.y + size.y);

        let center_x = (container.min.x + container.max.x) / 2;
        let (min_x, max_x) = match h_align {
            Align::Start => (container.min.x, container.min.x + size.x),
            Align::End => (container.max.x - size.x, container.max.x),
            Align::Center => (center_x - size.x / 2, center_x + size.x / 2),
        };

        let center_y = (container.min.y + container.max.y) / 2;
        let (min_y, max_y) = match v_align {
            Align::Start => (container.min.y, container.min.y + size.y),
            Align::End => (container.max.y - size.y, container.max.y),
            Align::Center => (center_y - size.y / 2, center_y + size.y / 2),
        };

        let me = Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        };

        assert!(container.contains_rect(me));

        me
    }

    pub fn draw(&self, color: crate::color::Color, ctx: &mut DrawCtx) {
        assert!(ctx.rect.contains_rect(*self));

        for x in self.min.x..self.max.x {
            for y in self.min.y..self.max.y {
                ctx.put(Point { x, y }, color);
            }
        }
    }

    pub fn draw_outline(&self, color: crate::color::Color, ctx: &mut DrawCtx) {
        assert!(ctx.rect.contains_rect(*self));

        for x in self.min.x..=self.max.x {
            ctx.put(Point { x, y: self.min.y }, color);
            ctx.put(Point { x, y: self.max.y }, color);
        }

        for y in self.min.y + 1..self.max.y {
            ctx.put(Point { x: self.min.x, y }, color);
            ctx.put(Point { x: self.max.x, y }, color);
        }
    }

    pub fn damage_outline(&self, sur: WlSurface) {
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

    pub fn width(&self) -> u32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> u32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Point {
        Point::new(self.width(), self.height())
    }

    pub fn center(&self) -> Point {
        Point {
            x: (self.min.x + self.max.x) / 2,
            y: (self.min.y + self.max.y) / 2,
        }
    }

    pub fn bounding(&self, other: Rect) -> Rect {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    pub fn smallest(&self, other: Rect) -> Rect {
        Self::new(self.min.max(other.min), self.max.min(other.max))
    }

    pub fn contains(&self, p: Point) -> bool {
        self.min.x <= p.x && p.x <= self.max.x && self.min.y <= p.y && p.y <= self.max.y
    }

    pub fn contains_rect(&self, r: Rect) -> bool {
        self.min.x <= r.min.x
            && self.min.y <= r.min.y
            && self.max.x >= r.max.x
            && self.max.y >= r.max.y
    }
}

impl From<rusttype::Rect<i32>> for Rect {
    fn from(val: rusttype::Rect<i32>) -> Self {
        Rect {
            min: val.min.into(),
            max: val.max.into(),
        }
    }
}

impl From<Rect> for rusttype::Rect<u32> {
    fn from(val: Rect) -> Self {
        rusttype::Rect {
            min: rusttype::point(val.min.x, val.min.y),
            max: rusttype::point(val.max.x, val.max.y),
        }
    }
}
