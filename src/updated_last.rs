use crate::draw::prelude::*;
use crate::log::*;
use crate::widget::{ClickType, Widget};

use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use rusttype::Font;
use std::marker::PhantomData;

pub struct UpdatedLast {
    lc: LC,
    time: DateTime<Utc>,
    text: TextBox,
}

impl UpdatedLast {
    pub fn builder() -> UpdatedLastBuilder<NeedsFont> {
        Default::default()
    }
}

impl Widget for UpdatedLast {
    fn lc(&self) -> &LC {
        &self.lc
    }
    fn area(&self) -> Rect {
        self.text.area()
    }
    fn h_align(&self) -> Align {
        self.text.h_align()
    }
    fn v_align(&self) -> Align {
        self.text.v_align()
    }
    fn desired_height(&self) -> u32 {
        self.text.desired_height()
    }
    fn desired_width(&self, height: u32) -> u32 {
        height * MAX_LABEL_LEN * 2 / 3
    }
    fn resize(&mut self, area: Rect) {
        self.text.resize(area);
    }
    fn should_redraw(&mut self) -> bool {
        self.text.set_text(&label_from_time(Utc::now() - self.time));
        self.text.should_redraw()
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.text.draw(ctx)
    }

    fn click(&mut self, _button: ClickType, _point: Point) -> Result<()> {
        Ok(())
    }

    fn motion(&mut self, _point: Point) -> Result<()> {
        Ok(())
    }
    fn motion_leave(&mut self, _point: Point) -> Result<()> {
        Ok(())
    }
}

use core::cmp::Ordering;
const MAX_LABEL_LEN: u32 = "59 Minutes Ago".len() as u32;
fn label_from_time(delta_time: TimeDelta) -> String {
    if delta_time.num_seconds() < 0 {
        return "The Future?".into();
    }

    let days = delta_time.num_days();
    if days > 14 {
        return "UPDATE NOW!".into();
    }
    match days.cmp(&1) {
        Ordering::Equal => return "1 Day Ago".into(),
        Ordering::Greater => return format!("{days} Days Ago"),
        Ordering::Less => {}
    }

    let hours = delta_time.num_hours();
    match hours.cmp(&1) {
        Ordering::Equal => return "1 Hour Ago".into(),
        Ordering::Greater => return format!("{hours} Hours Ago"),
        Ordering::Less => {}
    }

    let minutes = delta_time.num_minutes();
    match minutes.cmp(&1) {
        Ordering::Equal => return "1 Minute Ago".into(),
        Ordering::Greater => return format!("{minutes} Minutes Ago"),
        Ordering::Less => {}
    }

    "Now".into()
}

#[derive(Clone, Debug, Default)]
pub struct UpdatedLastBuilder<T> {
    font: Option<Font<'static>>,
    time_stamp: i64,
    desired_height: Option<u32>,
    h_align: Align,
    v_align: Align,
    fg: Color,
    bg: Color,

    _state: PhantomData<T>,
}

impl<T> UpdatedLastBuilder<T> {
    pub fn new() -> UpdatedLastBuilder<NeedsFont> {
        Default::default()
    }

    crate::builder_fields! {
        i64, time_stamp;
        u32, desired_height;
        Align, v_align h_align;
        Color, fg bg;
    }

    pub fn font(self, font: Font<'static>) -> UpdatedLastBuilder<HasFont> {
        UpdatedLastBuilder {
            _state: PhantomData,
            font: Some(font),

            time_stamp: self.time_stamp,
            desired_height: self.desired_height,
            h_align: self.h_align,
            v_align: self.v_align,
            fg: self.fg,
            bg: self.bg,
        }
    }
}

impl UpdatedLastBuilder<HasFont> {
    pub fn build(&self, lc: LC) -> UpdatedLast {
        info!(
            lc,
            ":: Initializing with height: {}",
            self.desired_height.unwrap_or(u32::MAX)
        );
        let font = self.font.clone().unwrap();

        let time = chrono::DateTime::from_timestamp(self.time_stamp, 0)
            .unwrap_or(chrono::DateTime::UNIX_EPOCH);

        let text = TextBox::builder()
            .font(font)
            .v_align(self.v_align)
            .h_align(self.h_align)
            .right_margin(self.desired_height.unwrap_or(0) / 5)
            .fg(self.fg)
            .bg(self.bg)
            .text("Default Text")
            .desired_text_height(self.desired_height.map(|s| s * 20 / 23).unwrap_or(u32::MAX))
            .build(lc.child("Text"));

        UpdatedLast { lc, time, text }
    }
}
