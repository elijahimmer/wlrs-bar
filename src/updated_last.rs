use crate::draw::prelude::*;
use crate::widget::{ClickType, Widget};

use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use rusttype::Font;

pub struct UpdatedLast<'font> {
    name: Box<str>,
    time: DateTime<Utc>,
    text: TextBox<'font>,
    last_text_set: Box<str>,
}

impl UpdatedLast<'_> {
    pub fn builder<'a>() -> UpdatedLastBuilder<'a> {
        Default::default()
    }
}

impl Widget for UpdatedLast<'_> {
    fn name(&self) -> &str {
        &self.name
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
        let now = Utc::now();

        let new_text = &label_from_time(now - self.time);

        *new_text != *self.last_text_set
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        let now = Utc::now();
        let new_text = &label_from_time(now - self.time);

        self.last_text_set = new_text.clone().into();
        self.text.set_text(new_text);

        self.text.draw(ctx)?;

        Ok(())
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
        core::cmp::Ordering::Equal => return "1 Day Ago".into(),
        core::cmp::Ordering::Greater => return format!("{days} Days Ago"),
        core::cmp::Ordering::Less => {}
    }

    let hours = delta_time.num_hours();
    match hours.cmp(&1) {
        core::cmp::Ordering::Equal => return "1 Hour Ago".into(),
        core::cmp::Ordering::Greater => return format!("{hours} Hours Ago"),
        core::cmp::Ordering::Less => {}
    }

    let minutes = delta_time.num_minutes();
    match minutes.cmp(&1) {
        core::cmp::Ordering::Equal => return "1 Minute Ago".into(),
        core::cmp::Ordering::Greater => return format!("{minutes} Minutes Ago"),
        core::cmp::Ordering::Less => {}
    }

    "Now".into()
}

#[derive(Clone, Debug, Default)]
pub struct UpdatedLastBuilder<'font> {
    font: Option<Font<'font>>,
    time_stamp: i64,
    desired_height: Option<u32>,
    h_align: Align,
    v_align: Align,
    fg: Color,
    bg: Color,
}

impl<'font> UpdatedLastBuilder<'font> {
    pub fn new() -> Self {
        Default::default()
    }

    crate::builder_fields! {
        Font<'font>, font;
        i64, time_stamp;
        u32, desired_height;
        Align, v_align h_align;
        Color, fg bg;
    }

    pub fn build(&self, name: &str) -> UpdatedLast<'font> {
        log::info!(
            "'{name}' :: Initializing with height: {}",
            self.desired_height.unwrap_or(u32::MAX)
        );
        let font = self
            .font
            .clone()
            .unwrap_or_else(|| panic!("'{}' A font must be provided", name));

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
            .build(&(name.to_owned() + " Text"));

        UpdatedLast {
            name: name.into(),
            time,
            text,
            last_text_set: "Default Text".into(),
        }
    }
}
