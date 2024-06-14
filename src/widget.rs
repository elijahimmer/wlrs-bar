use anyhow::Result;

use crate::draw::{DrawCtx, Point, Rect};

pub trait Widget {
    fn area(&self) -> Rect;
    fn desired_size(&self) -> Point;
    fn resize(&mut self, rect: Rect);
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()>;
}
