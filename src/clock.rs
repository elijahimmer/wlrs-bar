use crate::draw::prelude::*;
use crate::widget::{center_widgets, ClickType, Widget};

use anyhow::Result;
use chrono::Timelike;
use rusttype::Font;
use std::marker::PhantomData;

pub struct Clock {
    name: Box<str>,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,

    __hours: TextBox,
    spacer1: TextBox,
    minutes: TextBox,
    spacer2: TextBox,
    seconds: TextBox,
}

impl Clock {
    pub fn builder() -> ClockBuilder<NeedsFont> {
        Default::default()
    }
    fn update_time(&mut self) {
        let time = chrono::Local::now();

        //log::warn!(
        //    "'{}' update_time :: {}:{}:{}",
        //    self.name,
        //    time.hour(),
        //    time.minute(),
        //    time.second()
        //);
        self.__hours
            .set_text(&format2digits(time.hour().try_into().unwrap()));
        self.minutes
            .set_text(&format2digits(time.minute().try_into().unwrap()));
        self.seconds
            .set_text(&format2digits(time.second().try_into().unwrap()));
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

impl Widget for Clock {
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

    fn should_redraw(&mut self) -> bool {
        self.update_time();

        self.seconds.should_redraw()
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        inner_as_slice!(self mut).iter_mut().for_each(|w| {
            if w.should_redraw() {
                if let Err(err) = w.draw(ctx) {
                    log::warn!(
                        "'{}' | draw :: widget '{}' failed to draw. error={err}",
                        self.name,
                        w.name()
                    );
                }
            }
        });

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

fn format2digits(n: u8) -> Box<str> {
    let mut s = String::with_capacity(2);
    s.push((b'0' + (n / 10)) as char);
    s.push((b'0' + (n % 10)) as char);

    s.into()
}

#[derive(Clone, Debug, Default)]
pub struct ClockBuilder<T> {
    font: Option<Font<'static>>,
    desired_height: Option<u32>,
    h_align: Align,
    v_align: Align,
    number_fg: Color,
    spacer_fg: Color,
    bg: Color,

    _state: PhantomData<T>,
}

impl<T> ClockBuilder<T> {
    pub fn new() -> ClockBuilder<NeedsFont> {
        Default::default()
    }

    crate::builder_fields! {
        u32, desired_height;
        Align, v_align h_align;
        Color, number_fg spacer_fg bg;
    }

    pub fn font(self, font: Font<'static>) -> ClockBuilder<HasFont> {
        ClockBuilder {
            _state: PhantomData,
            font: Some(font),

            desired_height: self.desired_height,
            h_align: self.h_align,
            v_align: self.v_align,
            number_fg: self.number_fg,
            spacer_fg: self.spacer_fg,
            bg: self.bg,
        }
    }
}

impl ClockBuilder<HasFont> {
    pub fn build(&self, name: &str) -> Clock {
        let desired_height = self.desired_height.unwrap_or(u32::MAX / 2);
        log::info!("'{name}' :: Initializing with height: {desired_height}");
        let font = self
            .font
            .clone()
            .unwrap_or_else(|| panic!("'{}' A font should be provided", name));

        let time_builder = TextBox::builder()
            .font(font.clone())
            .text("00")
            .fg(self.number_fg)
            .bg(self.bg)
            .desired_text_height(desired_height)
            .desired_width(desired_height);

        let spacer_builder = TextBox::builder()
            .font(font)
            .text("")
            .fg(self.spacer_fg)
            .bg(self.bg)
            .desired_text_height(desired_height * 2 / 3)
            .h_margins(desired_height / 5)
            .v_align(Align::CenterAt(0.45));

        let __hours = time_builder.build(&(name.to_owned() + "   hours"));
        let minutes = time_builder.build(&(name.to_owned() + " minutes"));
        let seconds = time_builder.build(&(name.to_owned() + " seconds"));

        let spacer1 = spacer_builder.build(&(name.to_owned() + " spacer1"));
        let spacer2 = spacer_builder.build(&(name.to_owned() + " spacer2"));

        Clock {
            name: name.into(),
            desired_height,
            h_align: self.h_align,
            v_align: self.v_align,

            __hours,
            spacer1,
            minutes,
            spacer2,
            seconds,
            area: Default::default(),
        }
    }
}
