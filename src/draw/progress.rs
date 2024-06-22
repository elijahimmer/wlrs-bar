use super::prelude::*;
use crate::widget::{ClickType, PositionedWidget, Widget};
use anyhow::Result;

use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash, Default)]
pub enum RedrawState {
    #[default]
    None,
    /// redraw the entire thing
    Redraw,
    /// just add the next rows/columns of pixels
    Append(NonZeroU32),
}

/// A single character displayed as large as possible
pub struct Progress {
    name: Box<str>,

    filled_color: Color,
    unfilled_color: Color,
    bg: Color,

    fill_direction: Direction,

    /// the amount to fill starting for min_filled
    diff_filled: f32,
    /// lowest fill amount
    min_filled: f32,
    /// ratio of how much
    ratio_unfilled: f32,

    /// ratio of height to top_margin
    top_margin: f32,
    /// ratio of height to bottom_margin
    bottom_margin: f32,
    /// ratio of height to left_margin
    left_margin: f32,
    /// ratio of height to right_margin
    right_margin: f32,

    h_align: Align,
    v_align: Align,

    redraw: RedrawState,
    area: Rect,
    area_used: Rect,
    desired_height: u32,
    desired_width: u32,
}

impl Progress {
    pub fn builder() -> ProgressBuilder {
        ProgressBuilder::new()
    }

    pub fn set_progress(&mut self, progress: f32) {
        assert!(progress > self.min_filled);
        let progress = progress - self.min_filled;
        assert!(progress <= self.diff_filled);
        let ratio_unfilled = 1.0 - (progress / self.diff_filled);
        assert!((0.0..=1.0).contains(&ratio_unfilled));
        self.ratio_unfilled = ratio_unfilled;
    }

    pub fn set_filled_color(&mut self, c: Color) {
        if c != self.filled_color {
            self.redraw = RedrawState::Redraw;
            self.filled_color = c;
        }
    }

    pub fn set_unfilled_color(&mut self, c: Color) {
        if c != self.unfilled_color {
            self.redraw = RedrawState::Redraw;
            self.unfilled_color = c;
        }
    }

    pub fn set_bg(&mut self, bg: Color) {
        if bg != self.bg {
            self.redraw = RedrawState::Redraw;
            self.bg = bg;
        }
    }
}

impl Widget for Progress {
    fn name(&self) -> &str {
        &self.name
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn h_align(&self) -> Align {
        self.h_align
    }
    fn v_align(&self) -> Align {
        self.v_align
    }

    fn desired_height(&self) -> u32 {
        self.desired_height.saturating_add(self.v_margins())
    }

    fn desired_width(&self, _height: u32) -> u32 {
        self.desired_width.saturating_add(self.h_margins())
    }

    fn resize(&mut self, new_area: Rect) {
        #[cfg(feature = "progress-logs")]
        log::trace!("'{}' | resize :: new_area: {new_area}", self.name);
        self.area = new_area;
        self.redraw = RedrawState::Redraw;
        let max_area = new_area
            .shrink_top(self.top_margin())
            .shrink_bottom(self.bottom_margin())
            .shrink_left(self.left_margin())
            .shrink_right(self.right_margin());

        self.area_used = max_area.place_at(
            Point {
                x: self.desired_width.min(max_area.width()),
                y: self.desired_height.min(max_area.height()),
            },
            self.h_align,
            self.v_align,
        );

        #[cfg(feature = "progress-logs")]
        log::trace!("'{}' | resize :: area_used: {}", self.name, self.area_used);
    }

    fn should_redraw(&mut self) -> bool {
        self.redraw != RedrawState::None
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        assert!((0.0..=1.0).contains(&self.ratio_unfilled));
        self.redraw = RedrawState::None;

        //let redraw = if ctx.full_redraw {
        //    RedrawState::Redraw
        //} else {
        //    self.redraw
        //};

        //if let RedrawState::Append(lines) = redraw {
        //    todo!()
        //} else {
        self.area.draw_composite(self.bg, ctx);
        self.area_used.draw_composite(self.unfilled_color, ctx);

        let width_not_filled = (self.area_used.width() as f32 * self.ratio_unfilled) as u32;
        let height_not_filled = (self.area_used.height() as f32 * self.ratio_unfilled) as u32;

        let filled_area = match self.fill_direction {
            Direction::North => self.area_used.shrink_top(height_not_filled),
            Direction::South => self.area_used.shrink_bottom(height_not_filled),
            Direction::East => self.area_used.shrink_right(width_not_filled),
            Direction::West => self.area_used.shrink_left(width_not_filled),
        };

        filled_area.draw_composite(self.filled_color, ctx);

        #[cfg(feature = "progress-outlines")]
        self.area.draw_outline(super::color::PINE, ctx);
        #[cfg(feature = "progress-outlines")]
        self.area_used.draw_outline(super::color::IRIS, ctx);

        Ok(())
    }

    fn click(&mut self, _button: ClickType, _point: Point) -> Result<()> {
        todo!()
    }

    fn motion(&mut self, _point: Point) -> Result<()> {
        todo!()
    }

    fn motion_leave(&mut self, _point: Point) -> Result<()> {
        todo!()
    }
}

impl PositionedWidget for Progress {
    fn top_margin(&self) -> u32 {
        (self.area().height() as f32 * self.top_margin) as u32
    }
    fn bottom_margin(&self) -> u32 {
        (self.area().height() as f32 * self.bottom_margin) as u32
    }
    fn left_margin(&self) -> u32 {
        (self.area().width() as f32 * self.left_margin) as u32
    }
    fn right_margin(&self) -> u32 {
        (self.area().width() as f32 * self.right_margin) as u32
    }
}

#[derive(Clone)]
pub struct ProgressBuilder {
    filled_color: Color,
    unfilled_color: Color,
    bg: Color,

    fill_direction: Direction,

    /// height amoun
    ending_bound: f32,
    /// lowest fill amount
    starting_bound: f32,

    /// ratio of height to top_margin
    top_margin: f32,
    /// ratio of height to bottom_margin
    bottom_margin: f32,
    /// ratio of height to left_margin
    left_margin: f32,
    /// ratio of height to right_margin
    right_margin: f32,

    h_align: Align,
    v_align: Align,

    desired_height: u32,
    desired_width: u32,
}

impl ProgressBuilder {
    pub fn new() -> ProgressBuilder {
        Self {
            top_margin: 0.0,
            bottom_margin: 0.0,
            left_margin: 0.0,
            right_margin: 0.0,

            ending_bound: 0.0,
            starting_bound: 0.0,

            desired_height: u32::MAX,
            desired_width: u32::MAX,

            fill_direction: Default::default(),
            filled_color: Default::default(),
            unfilled_color: Default::default(),
            bg: Default::default(),

            h_align: Default::default(),
            v_align: Default::default(),
        }
    }

    crate::builder_fields! {
        u32, desired_height desired_width;
        f32, top_margin bottom_margin left_margin right_margin starting_bound ending_bound;
        Color, filled_color unfilled_color bg;
        Align, v_align h_align;
        Direction, fill_direction;
    }

    pub fn h_margins(mut self, margin: f32) -> Self {
        self.left_margin = margin / 2.0;
        self.right_margin = margin / 2.0;
        self
    }

    pub fn v_margins(mut self, margin: f32) -> Self {
        self.top_margin = margin / 2.0;
        self.bottom_margin = margin / 2.0;
        self
    }

    pub fn build(&self, name: &str) -> Progress {
        Progress {
            name: name.into(),

            filled_color: self.filled_color,
            unfilled_color: self.unfilled_color,
            bg: self.bg,

            fill_direction: self.fill_direction,

            diff_filled: self.ending_bound - self.starting_bound,
            min_filled: self.starting_bound,
            ratio_unfilled: 0.0,

            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            left_margin: self.left_margin,
            right_margin: self.right_margin,

            h_align: self.h_align,
            v_align: self.v_align,

            desired_height: self.desired_height,
            desired_width: self.desired_width,

            redraw: Default::default(),
            area: Default::default(),
            area_used: Default::default(),
        }
    }
}

impl Default for ProgressBuilder {
    fn default() -> Self {
        Self::new()
    }
}
