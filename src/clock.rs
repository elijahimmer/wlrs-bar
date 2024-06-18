use crate::color;
use crate::draw::{Align, DrawCtx, Rect, TextBox};
use crate::widget::{center_widgets, Widget};

use anyhow::Result;
use chrono::Timelike;

pub struct Clock<'a> {
    name: Box<str>,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,

    __hours: TextBox<'a>,
    spacer1: TextBox<'a>,
    minutes: TextBox<'a>,
    spacer2: TextBox<'a>,
    seconds: TextBox<'a>,
}

impl Clock<'_> {
    pub fn new<'a>(
        name: Box<str>,
        desired_height: u32,
        h_align: Align,
        v_align: Align,
    ) -> Clock<'a> {
        log::info!("'{name}' | new :: initializing with height: {desired_height}");

        let time_builder = TextBox::builder()
            .text("00".to_string())
            .fg(color::ROSE)
            .bg(color::SURFACE)
            .desired_text_height(desired_height);

        let spacer_builder = TextBox::builder()
            .text("".to_string())
            .fg(color::PINE)
            .bg(color::SURFACE)
            .desired_text_height(desired_height / 3 * 2)
            .h_margins(desired_height / 5)
            .v_align(Align::CenterAt(0.45));

        let __hours = time_builder.build((name.to_string() + "   hours").into());
        let minutes = time_builder.build((name.to_string() + " minutes").into());
        let seconds = time_builder.build((name.to_string() + " seconds").into());

        let spacer1 = spacer_builder.build((name.to_string() + " spacer1").into());
        let spacer2 = spacer_builder.build((name.to_string() + " spacer2").into());

        Clock {
            name,
            desired_height,
            h_align,
            v_align,

            __hours,
            spacer1,
            minutes,
            spacer2,
            seconds,
            area: Default::default(),
        }
    }

    fn update_time(&mut self) {
        let time = chrono::Local::now();
        self.__hours.set_text(&format2digits(time.hour() as u8));
        self.minutes.set_text(&format2digits(time.minute() as u8));
        self.seconds.set_text(&format2digits(time.second() as u8));
    }
}

macro_rules! inner_as_slice {
    ($s:ident) => {
        [
            &$s.minutes,
            &$s.spacer1,
            &$s.spacer2,
            &$s.seconds,
            &$s.__hours,
        ]
    };
    ($s:ident mut) => {
        [
            &mut $s.minutes,
            &mut $s.spacer1,
            &mut $s.spacer2,
            &mut $s.seconds,
            &mut $s.__hours,
        ]
    };
}

impl Widget for Clock<'_> {
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
        self.desired_height
    }

    fn desired_width(&self, height: u32) -> u32 {
        inner_as_slice!(self)
            .iter_mut()
            .fold(0, |acc, w| acc + w.desired_width(height))
    }

    fn resize(&mut self, area: Rect) {
        center_widgets(&mut inner_as_slice!(self mut), area);
        self.area = area;
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.update_time();
        inner_as_slice!(self mut).iter_mut().for_each(|w| {
            if let Err(err) = w.draw(ctx) {
                log::warn!(
                    "'{}' | draw :: widget '{}' failed to draw. error={err}",
                    self.name,
                    w.name()
                );
            }
        });

        Ok(())
    }
}

fn format2digits(n: u8) -> Box<str> {
    let mut s = String::with_capacity(2);
    s.push((b'0' + (n / 10)) as char);
    s.push((b'0' + (n % 10)) as char);

    s.into()
}
