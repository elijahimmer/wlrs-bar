use crate::draw::prelude::*;
use anyhow::Result;

pub trait Widget {
    fn name(&self) -> &str;
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

/// places widgets from the center propagating out,
/// scaling all down by the same ratio if needed.
/// the widgets are places the center first, then left and right.
/// if there is a even amount, 2 are placed with edges on the center line.
pub fn center_widgets(widgets: &mut [&mut impl Widget], area: Rect) {
    let (width_max, height_max) = (area.width(), area.height());
    log::trace!("center_widgets :: {area}");
    let mut widths: Vec<_> = widgets
        .iter()
        .map(|w| w.desired_width(height_max))
        .collect();
    let width_total: u32 = widths.iter().sum();

    if width_total > width_max {
        let ratio = width_max / width_total;

        widths.iter_mut().for_each(|w| (*w) *= ratio);
    }

    let mut iter = (0..)
        .map(|i| i % 2 == 0)
        .zip(widgets.iter_mut().zip(widths.iter()));

    let mut left = Rect::new(
        area.min,
        area.min
            + Point {
                x: width_max / 2,
                y: height_max,
            },
    );
    let mut right = Rect::new(
        area.min
            + Point {
                x: width_max / 2,
                y: 0,
            },
        area.max,
    );
    log::trace!("center_widgets :: left: {left}, right: {right}");

    if widths.len() % 2 == 1 {
        // is odd
        let (_, (widget, &width)) = iter.next().unwrap();
        let rect = area.place_at(
            Point {
                x: width,
                y: height_max,
            },
            Align::Center,
            Align::Center,
        );
        log::trace!("center_widgets :: rect: {rect}, width: {width}");

        widget.resize(rect);

        left.max.x -= rect.width() / 2;
        right.min.x += rect.width() / 2;
        assert!(left.min.x <= left.max.x);
        assert!(right.min.x <= right.max.x);
    };
    log::trace!("center_widgets :: left: {left}, right: {right}");

    iter.for_each(|(go_left, (widget, &width))| {
        let rect = if go_left {
            left.place_at(
                Point {
                    x: width,
                    y: height_max,
                },
                Align::End,
                Align::Center,
            )
        } else {
            right.place_at(
                Point {
                    x: width,
                    y: height_max,
                },
                Align::Start,
                Align::Center,
            )
        };

        widget.resize(rect);

        if go_left {
            left.max.x -= rect.width();
        } else {
            right.min.x += rect.width();
        }
    });
}

#[macro_export]
macro_rules! builder_fields {
    ($($t: ty, $($n: ident)+;)+) => ($($(
        pub fn $n(mut self, $n: $t) -> Self {
            self.$n = $n.into();
            self
        }
    )*)*)
}
