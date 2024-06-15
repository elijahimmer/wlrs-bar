use anyhow::Result;
use chrono::Timelike;

use crate::color;
use crate::draw::{DrawCtx, Rect, TextBox};
use crate::widget::{center_widgets, Widget};

pub const NUM_CHARS: u32 = 8;

pub struct Clock<'a> {
    pub name: String,
    desired_height: f32,
    area: Rect<f32>,

    hours: TextBox<'a>,
    spacer1: TextBox<'a>,
    minutes: TextBox<'a>,
    spacer2: TextBox<'a>,
    seconds: TextBox<'a>,
}

impl Clock<'_> {
    pub fn new<'a>(name: String, desired_height: f32) -> Clock<'a> {
        log::info!("Initalizing Clock");

        let time_builder = TextBox::builder()
            .text("00".to_string())
            .fg(*color::ROSE)
            .desired_height(desired_height);

        let hours = time_builder.clone().build("clock   hours".to_owned());
        let minutes = time_builder.clone().build("clock minutes".to_owned());
        let seconds = time_builder.clone().build("clock seconds".to_owned());

        let spacer_builder = TextBox::builder()
            .text("".to_string())
            .desired_height(desired_height / 2.0)
            .fg(*color::PINE);

        let spacer1 = spacer_builder.clone().build("clock spacer1".to_owned());
        let spacer2 = spacer_builder.clone().build("clock spacer2".to_owned());

        Clock {
            name,
            desired_height,

            hours,
            spacer1,
            minutes,
            spacer2,
            seconds,
            area: Default::default(),
        }
    }

    fn update_time(&mut self) {
        let time = chrono::Local::now();
        self.hours.set_text(format2digits(time.hour() as u8));
        self.minutes.set_text(format2digits(time.minute() as u8));
        self.seconds.set_text(format2digits(time.second() as u8));
    }
}

impl Widget for Clock<'_> {
    fn name(&self) -> &String {
        &self.name
    }

    fn area(&self) -> Rect<f32> {
        self.area
    }

    fn desired_height(&self) -> f32 {
        self.desired_height
    }

    fn desired_width(&self, height: f32) -> f32 {
        self.hours.desired_width(height)
            + self.spacer1.desired_width(height)
            + self.minutes.desired_width(height)
            + self.spacer2.desired_width(height)
            + self.seconds.desired_width(height)
    }

    fn resize(&mut self, area: Rect<f32>) {
        center_widgets(
            &mut [
                &mut self.minutes,
                &mut self.spacer1,
                &mut self.spacer2,
                &mut self.seconds,
                &mut self.hours,
            ],
            area,
        );
        self.area = area;
    }
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.update_time();

        self.hours.draw(ctx)?;
        self.spacer1.draw(ctx)?;
        self.minutes.draw(ctx)?;
        self.spacer2.draw(ctx)?;
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
