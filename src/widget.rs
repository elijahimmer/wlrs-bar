use anyhow::Result;

use crate::draw::{Align, DrawCtx, Point, Rect};

pub trait Widget {
    fn name(&self) -> &str;
    fn area(&self) -> Rect<f32>;
    fn h_align(&self) -> Align;
    fn v_align(&self) -> Align;
    fn desired_height(&self) -> f32;
    fn desired_width(&self, height: f32) -> f32;

    fn resize(&mut self, rect: Rect<f32>);
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()>;
}

pub trait PositionedWidget {
    fn top_margin(&self) -> f32;
    fn bottom_margin(&self) -> f32;
    fn left_margin(&self) -> f32;
    fn right_margin(&self) -> f32;

    fn v_margins(&self) -> f32 {
        self.top_margin() + self.bottom_margin()
    }

    fn h_margins(&self) -> f32 {
        self.left_margin() + self.right_margin()
    }
}

// places widgets from the center propagating out,
// scaling all down by the same ratio if needed.
// the widgets are places the center first, then left and right.
// if there is a even amount, 2 are placed with edges on the center line.
pub fn center_widgets(widgets: &mut [&mut impl Widget], area: Rect<f32>) {
    let (width_max, height_max) = (area.width(), area.height());
    let mut widths: Vec<_> = widgets
        .iter()
        .map(|w| w.desired_width(height_max))
        .collect();
    let width_total: f32 = widths.iter().fold(0.0, |acc, new| acc + new);

    if width_total > width_max {
        let ratio = width_max / width_total;

        widths.iter_mut().for_each(|w| (*w) *= ratio);
    }

    let mut iter = (0..)
        .map(|i| i % 2 == 0)
        .zip(widgets.iter_mut().zip(widths.iter()));

    let mut left = Rect::new(area.min, area.min + Point::new(width_max / 2.0, height_max));
    let mut right = Rect::new(area.min + Point::new(width_max / 2.0, 0.0), area.max);

    if widths.len() % 2 == 1 {
        // is odd
        let (_, (widget, &width)) = iter.next().unwrap();
        let rect = area.place_at(Point::new(width, height_max), Align::Center, Align::Center);

        widget.resize(rect);

        left.max.x -= rect.width() / 2.0;
        right.min.x += rect.width() / 2.0;
        debug_assert!(left.min.x < left.max.x);
        debug_assert!(right.min.x < right.max.x);
    };

    iter.for_each(|(go_left, (widget, &width))| {
        let rect = if go_left {
            left.place_at(Point::new(width, height_max), Align::End, Align::Center)
        } else {
            right.place_at(Point::new(width, height_max), Align::Start, Align::Center)
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
    ($($t: ty, $($n: ident)+),+) => ($($(
        pub fn $n(mut self, $n: $t) -> Self {
            self.$n = $n;
            self
        }
    )*)*)
}
