pub mod place_widgets;
pub use place_widgets::*;

pub mod container;

use crate::draw::prelude::*;
use crate::log::*;
use anyhow::Result;

pub trait Widget {
    fn lc(&self) -> &LC;
    fn area(&self) -> Rect;
    fn h_align(&self) -> Align;
    fn v_align(&self) -> Align;
    fn desired_height(&self) -> u32;
    fn desired_width(&self, height: u32) -> u32;

    fn resize(&mut self, rect: Rect);
    fn should_redraw(&mut self) -> bool;
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()>;

    fn click(&mut self, button: ClickType, point: Point) -> Result<()>;
    fn motion(&mut self, point: Point) -> Result<()>;
    fn motion_leave(&mut self, point: Point) -> Result<()>;
}

pub trait PositionedWidget {
    fn top_margin(&self) -> u32;
    fn bottom_margin(&self) -> u32;
    fn left_margin(&self) -> u32;
    fn right_margin(&self) -> u32;

    fn v_margins(&self) -> u32 {
        self.top_margin() + self.bottom_margin()
    }

    fn h_margins(&self) -> u32 {
        self.left_margin() + self.right_margin()
    }

    fn margins(&self) -> Point {
        Point {
            x: self.h_margins(),
            y: self.v_margins(),
        }
    }
}

// TODO: Find a new home for this...
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClickType {
    LeftClick,
    RightClick,
    MiddleClick,
    Other,
}

impl ClickType {
    pub fn new(button: u32) -> Self {
        match button {
            272 => Self::LeftClick,
            273 => Self::RightClick,
            274 => Self::MiddleClick,
            _ => Self::Other,
        }
    }
}

//pub trait Builder {
//    type Widget;
//    fn new() -> Self;
//    fn build(&self, name: &str) -> Self::Widget;
//}

#[macro_export]
macro_rules! builder_fields {
    ($($t: ty, $($n: ident)+;)+) => ($($(
        pub fn $n(mut self, $n: $t) -> Self {
            self.$n = $n.into();
            self
        }
    )*)*)
}
