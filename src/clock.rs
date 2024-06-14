use anyhow::Result;
use chrono::Timelike;
use rusttype::Scale;

use crate::color;
use crate::draw::{Align, DrawCtx, Point, Rect, TextBox};
use crate::widget::Widget;

pub const NUM_CHARS: u32 = 8;

#[derive(Clone)]
pub struct Clock {
    pub scale: Scale,
    pub rect: Rect,
    pub hours: TextBox,
    pub spacer1: TextBox,
    pub minutes: TextBox,
    pub spacer2: TextBox,
    pub seconds: TextBox,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    pub fn new() -> Self {
        log::info!("Initalizing Clock");
        let scale = Scale::uniform(crate::app::HEIGHT as f32);

        let fg = *color::ROSE;
        let bg = *color::SURFACE;

        let hours = TextBox::new("00".to_string(), scale, fg, bg);
        let minutes = TextBox::new("00".to_string(), scale, fg, bg);
        let seconds = TextBox::new("00".to_string(), scale, fg, bg);

        let spacer1 = TextBox::new("".to_string(), scale, fg, bg);
        let spacer2 = TextBox::new("".to_string(), scale, fg, bg);

        Self {
            scale,
            rect: Rect::default(),
            hours,
            spacer1,
            minutes,
            spacer2,
            seconds,
        }
    }

    fn update_time(&mut self) {
        let time = chrono::Local::now();
        self.hours.text = format2digits(time.hour() as u8);
        self.minutes.text = format2digits(time.minute() as u8);
        self.seconds.text = format2digits(time.second() as u8);
    }
}

impl Widget for Clock {
    fn area(&self) -> Rect {
        self.rect
    }

    fn desired_size(&self) -> Point {
        Point::new(self.scale.x as u32 * NUM_CHARS / 2, self.scale.y as u32)
    }

    fn resize(&mut self, rect: Rect) {
        log::info!("Moving Clock from: {:?} to {rect:?}", self.rect);
        self.rect = rect;

        let text_size = self.hours.desired_size();

        debug_assert_eq!(text_size, self.minutes.desired_size());
        debug_assert_eq!(text_size, self.seconds.desired_size());

        let hours_rect = Rect::place_at(rect, text_size, Align::Start, Align::Center);
        let minutes_rect = Rect::place_at(rect, text_size, Align::Center, Align::Center);
        let seconds_rect = Rect::place_at(rect, text_size, Align::End, Align::Center);

        let spacer1_rect = hours_rect.smallest(minutes_rect);
        let spacer2_rect = minutes_rect.smallest(seconds_rect);

        self.hours.resize(hours_rect);
        self.spacer1.resize(spacer1_rect);
        self.minutes.resize(minutes_rect);
        self.spacer2.resize(spacer2_rect);
        self.seconds.resize(seconds_rect);
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.update_time();

        self.hours.draw(ctx)?;
        self.minutes.draw(ctx)?;
        self.seconds.draw(ctx)?;

        Ok(())
    }
}

fn format2digits(n: u8) -> String {
    let mut s = String::with_capacity(2);
    s.push((b'0' + (n / 10)) as char);
    s.push((b'0' + (n % 10)) as char);

    s
}
