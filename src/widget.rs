use anyhow::Result;

pub trait Widget {
    fn new(dimensions: Rect) -> Self
    where
        Self: Sized;
    fn recommended_size(container: Rect) -> Point
    where
        Self: Sized;
    fn desired_size(&self, container: Rect) -> Point;
    fn draw(&mut self, canvas: &mut [u8], canvas_size: Point) -> Result<()>;
    fn update_dimensions(&mut self, dimensions: Rect);
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl From<(u32, u32)> for Point {
    fn from(v: (u32, u32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<Point> for rusttype::Point<f32> {
    fn from(val: Point) -> Self {
        rusttype::Point {
            x: val.x as f32,
            y: val.y as f32,
        }
    }
}

impl From<Point> for rusttype::Point<u32> {
    fn from(val: Point) -> Self {
        rusttype::Point { x: val.x, y: val.y }
    }
}

fn cmp<T: std::cmp::PartialOrd>(a: T, b: T) -> (T, T) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }

    pub fn extend(self, x: u32, y: u32) -> Rect {
        let (min_x, max_x) = cmp(self.x, x);
        let (min_y, max_y) = cmp(self.y, y);

        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

// which edge to align to
#[derive(Clone, Copy, Debug)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn from_size(container: Rect, desired_size: Point, h_align: Align, v_align: Align) -> Self {
        debug_assert!(container.width >= desired_size.x);
        debug_assert!(container.height >= desired_size.y);

        let x = match h_align {
            Align::Start => container.x,
            Align::End => container.x - desired_size.x,
            Align::Center => container.x + (container.width - desired_size.x) / 2,
        };

        let y = match v_align {
            Align::Start => container.y,
            Align::End => container.y - desired_size.y,
            Align::Center => container.y + (container.height - desired_size.y) / 2,
        };

        debug_assert!(container.x <= x && x <= container.x + container.width);
        debug_assert!(container.y <= y && y <= container.y + container.height);

        debug_assert!(x + desired_size.x <= container.x + container.width);
        debug_assert!(y + desired_size.y <= container.y + container.height);

        Self {
            x,
            y,
            width: desired_size.x,
            height: desired_size.y,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        self.x <= p.x && self.x + self.width >= p.x && self.y <= p.y && self.y + self.height >= p.y
    }

    pub fn fill(&self, color: crate::color::Color, canvas: &mut [u8], canvas_size: Point) {
        debug_assert!(self.x + self.width <= canvas_size.x);
        debug_assert!(self.y + self.height <= canvas_size.y);

        let argb = color.argb8888();
        for x in self.x..self.x + self.width {
            for y in self.y..self.y + self.height {
                let idx: usize = 4 * (x + y * canvas_size.x) as usize;

                let array: &mut [u8; 4] = (&mut canvas[idx..idx + 4]).try_into().unwrap();
                *array = argb;
            }
        }
    }
}

impl From<rusttype::Rect<i32>> for Rect {
    fn from(val: rusttype::Rect<i32>) -> Self {
        Rect {
            x: val.min.x as u32,
            y: val.min.y as u32,
            width: (val.max.x - val.min.x) as u32,
            height: (val.max.y - val.min.y) as u32,
        }
    }
}

impl From<Rect> for rusttype::Rect<u32> {
    fn from(val: Rect) -> Self {
        rusttype::Rect {
            min: rusttype::point(val.x, val.y),
            max: rusttype::point(val.x + val.width, val.y + val.height),
        }
    }
}
