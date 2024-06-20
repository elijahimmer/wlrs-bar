use crate::draw::prelude::*;
use crate::widget::{ClickType, Widget};

use anyhow::Result;

// TODO: I should make this not hard coded and read all of them.
//const BATTERY_FOLDER: &str = "/sys/class/power_supply/BAT0";
//const ENERGY_FULL_FILE: &str = concatcp!(BATTERY_FOLDER, "/energy_full");
//const BATTERY_STATUS_FILE: &str =
//const ENERGY_NOW_FILE: &str = concatcp!(BATTERY_FOLDER, "/energy_now");
//const MAX_TRIES: usize = 10;

pub struct Battery<'a> {
    name: Box<str>,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,

    widgets: [TextBox<'a>; 2],
}

impl Battery<'_> {
    pub fn builder() -> BatteryBuilder {
        BatteryBuilder::new()
    }
}

impl Widget for Battery<'_> {
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
        self.widgets[0].desired_width(height) + self.widgets[1].desired_width(height) / 2
    }

    fn resize(&mut self, area: Rect) {
        self.widgets.iter_mut().for_each(|w| w.resize(area));
        self.area = area;
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        //self.widgets[0].draw(ctx).unwrap();
        //self.widgets[1].draw(ctx).unwrap();
        self.widgets.iter_mut().for_each(|w| {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct BatteryBuilder {
    desired_height: Option<u32>,
    desired_width: Option<u32>,
    h_align: Align,
    v_align: Align,

    bg: Color,
    full_fg: Color,
    charging_fg: Color,
    ok_fg: Color,
    warn_fg: Color,
}

impl BatteryBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    crate::builder_fields! {
        u32, desired_height desired_width;
        Align, v_align h_align;
        Color, bg full_fg charging_fg ok_fg warn_fg;

    }

    pub fn build<'a>(&self, name: &str) -> Battery<'a> {
        let desired_height = self.desired_height.unwrap_or(u32::MAX / 2);
        log::info!("'{name}' :: Initializing with height: {desired_height}");

        let battery_textbox = TextBox::builder()
            .text("")
            .fg(crate::draw::color::PINE)
            .bg(self.bg)
            .desired_text_height(desired_height * 8 / 10)
            .h_align(Align::End)
            .v_align(Align::Center)
            .right_margin(desired_height / 3)
            .build(&(name.to_owned() + " Outline"));

        let charging_textbox = TextBox::builder()
            .text("󱐋")
            .fg(self.charging_fg)
            .bg(crate::draw::color::CLEAR)
            .h_align(Align::End)
            .v_align(Align::Center)
            .right_margin(desired_height / 10)
            .build(&(name.to_owned() + " Charging"));

        Battery {
            name: name.into(),
            desired_height,
            h_align: self.h_align,
            v_align: self.v_align,

            widgets: [battery_textbox, charging_textbox],
            area: Default::default(),
        }
    }
}

pub fn read_trim(path: &str) -> Result<Box<str>> {
    Ok(std::fs::read_to_string(path).map(|s| s.trim().into())?)
}

pub fn read_f64(path: &str) -> Result<f64> {
    Ok(std::fs::read_to_string(path)?.trim().parse::<f64>()?)
}

fn get_battery_info(battery_folder: &str) -> Result<(f64, Box<str>)> {
    let status = {
        let s: Box<str> = read_trim(&(battery_folder.to_owned() + "/status"))?;

        if *s == *"Not charging" {
            "Full".into()
        } else {
            s
        }
    };

    let energy = read_f64(&(battery_folder.to_owned() + "/energy_now"))?;

    Ok((energy, status))
}
