pub mod place_widgets;
pub use place_widgets::*;

pub mod container;

use crate::draw::prelude::*;
use crate::log::*;
use anyhow::Result;

pub trait Widget {
    /// Returns what context you should log for the widget
    fn lc(&self) -> &LC;

    /// Returns the Area that widget uses
    fn area(&self) -> Rect;

    /// Returns the horizontal alignment
    fn h_align(&self) -> Align;

    /// Returns the vertical alignment
    fn v_align(&self) -> Align;

    /// Returns the desired height of the widget
    fn desired_height(&self) -> u32;

    /// Returns the desired width of the widget
    fn desired_width(&self, height: u32) -> u32;

    /// Force the widget to use the new area given.
    fn resize(&mut self, rect: Rect);

    /// Whether or not the widget should be redrawn
    fn should_redraw(&mut self) -> bool;

    /// Draw the widget in the DrawCtx
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()>;

    /// Clicks the widget at a specific (global) point
    fn click(&mut self, button: ClickType, point: Point) -> Result<()>;

    /// Says that the cursor was moved into or within the widget
    fn motion(&mut self, point: Point) -> Result<()>;

    /// Says the cursor left the widget
    fn motion_leave(&mut self, point: Point) -> Result<()>;
}

pub trait PositionedWidget {
    /// Return the top internal margin of the widget
    fn top_margin(&self) -> u32;

    /// Return the bottom internal margin of the widget
    fn bottom_margin(&self) -> u32;

    /// Return the left internal margin of the widget
    fn left_margin(&self) -> u32;

    /// Return the right internal margin of the widget
    fn right_margin(&self) -> u32;

    /// Return the combined top and bottom internal margin of the widget
    fn v_margins(&self) -> u32 {
        self.top_margin() + self.bottom_margin()
    }

    /// Return the combined left and right internal margin of the widget
    fn h_margins(&self) -> u32 {
        self.left_margin() + self.right_margin()
    }

    /// Returns the x, y pair of the internal margins.
    fn margins(&self) -> Point {
        Point {
            x: self.h_margins(),
            y: self.v_margins(),
        }
    }
}

// TODO: Find a new home for this...
/// The click event type
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClickType {
    LeftClick,
    RightClick,
    MiddleClick,
    Other,
}

impl ClickType {
    /// turn a input code into a click event Assumes it is a click event.
    pub fn new(button: u32) -> Self {
        match button {
            272 => Self::LeftClick,
            273 => Self::RightClick,
            274 => Self::MiddleClick,
            _ => Self::Other,
        }
    }
}

/// Automatically makes the boilerplate constructor setters
/// The syntax is the type followed by a ',', then each of the fields of that type.
/// separate each of these lists by a ';'
/// So an example from container is
/// ```
/// impl ContainerBuilder {
/// // ...
///     crate::builder_fields! {
///         Align, v_align h_align inner_h_align;
/// //      ^^^^^  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// //      ^- types   ^- fields
///         u32, desired_height desired_width;
///     }
/// // ... (other fields)
/// }
/// ```
#[macro_export]
macro_rules! builder_fields {
    ($($t: ty, $($n: ident)+;)+) => ($($(
        pub fn $n(mut self, $n: $t) -> Self {
            self.$n = $n.into();
            self
        }
    )*)*)
}
