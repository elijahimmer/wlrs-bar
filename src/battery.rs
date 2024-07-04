use crate::draw::prelude::*;
use crate::log::*;
use crate::widget::{ClickType, Widget};

use anyhow::Result;
use rusttype::Font;
use std::marker::PhantomData;
use std::path::PathBuf;

// TODO: I should make this not hard coded and read all of them.
pub const DEFAULT_BATTERY_PATH: &str = "/sys/class/power_supply/BAT0";

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd)]
pub enum BatteryStatus {
    Full,
    Charging,
    #[default]
    Normal,
    Warn,
    Critical,
}

pub struct Battery {
    lc: LC,
    battery_path: PathBuf,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,

    battery: Icon,
    charging: Icon,
    progress: Progress,

    status: BatteryStatus,

    bg_color: Color,
    full_color: Color,
    charging_color: Color,
    normal_color: Color,
    warn_color: Color,
    critical_color: Color,
}

impl Battery {
    pub fn builder() -> BatteryBuilder<NeedsFont> {
        BatteryBuilder::<NeedsFont>::new()
    }

    pub fn update(&mut self) -> Result<()> {
        let mut energy_full_file = self.battery_path.clone();
        energy_full_file.push("energy_full");
        let mut energy_now_file = self.battery_path.clone();
        energy_now_file.push("energy_now");
        let mut status_file = self.battery_path.clone();
        status_file.push("status");

        let full: f32 = std::fs::read_to_string(&energy_full_file)?.trim().parse()?;
        let now: f32 = std::fs::read_to_string(&energy_now_file)?.trim().parse()?;

        let charge = (now / full).clamp(0.0, 1.0);

        let status = std::fs::read_to_string(status_file)?;

        // TODO: Make sure these actually make sense. (and exist)
        let status = match status.trim() {
            "Discharging" if charge < 0.1 => BatteryStatus::Critical,
            "Discharging" if charge < 0.25 => BatteryStatus::Warn,
            "Discharging" => BatteryStatus::Normal,
            "Critical" => BatteryStatus::Critical,
            "Not charging" | "Full" => BatteryStatus::Full,
            "Charging" if charge > 0.95 => BatteryStatus::Full,
            "Charging" => BatteryStatus::Charging,
            "Warn" => BatteryStatus::Warn,
            _ => {
                log::warn!("{} | update :: unknown battery status: '{status}'", self.lc);
                BatteryStatus::Normal
            }
        };

        if status != self.status {
            let c = match status {
                BatteryStatus::Full => self.full_color,
                BatteryStatus::Charging => self.charging_color,
                BatteryStatus::Normal => self.normal_color,
                BatteryStatus::Warn => self.warn_color,
                BatteryStatus::Critical => self.critical_color,
            };

            self.progress.set_filled_color(c);
            self.battery.set_fg(c);
            self.status = status;
            //log::trace!("{} | update :: color: {c}", self.lc);
        }

        self.progress.set_progress(charge);

        Ok(())
    }
}

impl Widget for Battery {
    fn lc(&self) -> &LC {
        &self.lc
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
        self.battery.desired_width(height)
    }

    fn resize(&mut self, area: Rect) {
        self.battery.resize(area);
        self.charging.resize(area);
        self.progress.resize(self.battery.area_used());
        self.area = area;
    }

    fn should_redraw(&mut self) -> bool {
        self.update().unwrap();

        self.progress.should_redraw()
            || self.battery.should_redraw()
            || if self.status == BatteryStatus::Charging {
                self.charging.should_redraw()
            } else {
                false
            }
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        //if self.progress.should_redraw() {
        self.area.draw(self.bg_color, ctx);
        self.battery.draw(ctx)?;
        self.progress.draw(ctx)?;
        log::trace!("status: {:?}", self.status);
        if self.status == BatteryStatus::Charging {
            self.charging.draw(ctx)?;
        }
        //}

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

#[derive(Clone, Debug, Default)]
pub struct BatteryBuilder<T> {
    font: Option<Font<'static>>,
    desired_height: Option<u32>,
    desired_width: Option<u32>,
    battery_path: Option<PathBuf>,
    h_align: Align,
    v_align: Align,

    bg: Color,
    full_color: Color,
    charging_color: Color,
    normal_color: Color,
    warn_color: Color,
    critical_color: Color,

    _state: PhantomData<T>,
}

impl<T> BatteryBuilder<T> {
    pub fn new() -> BatteryBuilder<NeedsFont> {
        Default::default()
    }

    crate::builder_fields! {
        Color, bg full_color charging_color normal_color warn_color critical_color;
        u32, desired_height desired_width;
        Align, v_align h_align;
        Option<PathBuf>, battery_path;
    }

    pub fn font(self, font: Font<'static>) -> BatteryBuilder<HasFont> {
        BatteryBuilder {
            _state: PhantomData,
            font: Some(font),

            h_align: self.h_align,
            v_align: self.v_align,

            bg: self.bg,
            full_color: self.full_color,
            charging_color: self.charging_color,
            normal_color: self.normal_color,
            warn_color: self.warn_color,
            critical_color: self.critical_color,

            battery_path: self.battery_path,
            desired_height: self.desired_height,
            desired_width: self.desired_width,
        }
    }
}

impl BatteryBuilder<HasFont> {
    pub fn build(&self, lc: LC) -> Result<Battery> {
        let battery_path = self
            .battery_path
            .clone()
            .unwrap_or_else(|| DEFAULT_BATTERY_PATH.into());

        assert!(battery_path.is_absolute());
        let battery_path = std::fs::canonicalize(&battery_path).unwrap_or(battery_path);

        // should error if the path doesn't exist
        _ = std::fs::read_dir(&battery_path)?;

        let desired_height = self.desired_height.unwrap_or(u32::MAX / 2);
        info!(lc, ":: Initializing with height: {desired_height}");
        let font = self.font.clone().unwrap();

        let battery = Icon::builder()
            .font(font.clone())
            .icon('')
            .fg(self.normal_color)
            .bg(color::CLEAR)
            .h_align(Align::End)
            .v_align(Align::Center)
            .right_margin(0.12)
            .left_margin(0.1)
            .v_margins(0.1)
            .build(lc.child("Outline"));

        let charging = Icon::builder()
            .font(font)
            .icon('󱐋')
            .fg(self.charging_color)
            .bg(color::CLEAR)
            .h_align(Align::End)
            .v_align(Align::Center)
            .right_margin(0.02)
            .build(lc.child("Charging"));

        let progress = Progress::builder()
            .top_margin(0.2)
            .bottom_margin(0.2)
            .left_margin(0.12)
            .right_margin(0.12)
            .starting_bound(0.0)
            .ending_bound(1.0)
            .fill_direction(Direction::East)
            .filled_color(self.normal_color)
            .unfilled_color(color::CLEAR)
            .bg(color::CLEAR)
            .build(lc.child("Progress"));

        Ok(Battery {
            lc,
            battery_path,
            desired_height,
            h_align: self.h_align,
            v_align: self.v_align,

            bg_color: self.bg,
            full_color: self.full_color,
            charging_color: self.charging_color,
            normal_color: self.normal_color,
            warn_color: self.warn_color,
            critical_color: self.critical_color,

            battery,
            charging,
            progress,

            area: Default::default(),
            status: Default::default(),
        })
    }
}
