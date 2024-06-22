use crate::draw::prelude::*;
use crate::widget::{ClickType, Widget};

use anyhow::Result;
use std::path::{Path, PathBuf};

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

pub struct Battery<'a> {
    name: Box<str>,
    battery_path: PathBuf,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,

    battery: Icon<'a>,
    charging: Icon<'a>,
    progress: Progress,

    status: BatteryStatus,

    full_color: Color,
    charging_color: Color,
    normal_color: Color,
    warn_color: Color,
    critical_color: Color,
}

impl Battery<'_> {
    pub fn builder() -> BatteryBuilder {
        BatteryBuilder::new()
    }

    pub fn update(&mut self) -> Result<()> {
        let mut energy_full_file = self.battery_path.clone();
        energy_full_file.push("/energy_full");
        let mut energy_now_file = self.battery_path.clone();
        energy_now_file.push("/energy_now");
        let mut status_file = self.battery_path.clone();
        status_file.push("/status");

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
            "Charging" => BatteryStatus::Charging,
            "Warn" => BatteryStatus::Warn,
            "Not Charging" | "Full" => BatteryStatus::Full,
            _ => {
                log::warn!(
                    "'{}' | update :: unknown battery status: '{status}'",
                    self.name
                );
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
        }

        self.progress.set_progress(charge);

        Ok(())
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
        self.battery.desired_width(height)
    }

    fn resize(&mut self, area: Rect) {
        self.battery.resize(area);
        self.charging.resize(area);
        self.progress.resize(area);
        self.area = area;
    }

    fn should_redraw(&mut self) -> bool {
        self.progress.should_redraw()
            || self.battery.should_redraw()
            || if self.status == BatteryStatus::Charging {
                self.charging.should_redraw()
            } else {
                false
            }
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.update()?;
        if self.progress.should_redraw() {
            self.battery.draw(ctx)?;
            self.progress.draw(ctx)?;
            if self.status == BatteryStatus::Charging {
                self.charging.draw(ctx)?;
            }
        }

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
pub struct BatteryBuilder {
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
}

impl BatteryBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    crate::builder_fields! {
        Color, bg full_color charging_color normal_color warn_color critical_color;
        u32, desired_height desired_width;
        Align, v_align h_align;
        Option<PathBuf>, battery_path;
    }

    pub fn build<'a>(&self, name: &str) -> Result<Battery<'a>> {
        let desired_height = self.desired_height.unwrap_or(u32::MAX / 2);
        log::info!("'{name}' :: Initializing with height: {desired_height}");

        let battery = Icon::builder()
            .icon('')
            .fg(self.normal_color)
            .bg(color::CLEAR)
            .h_align(Align::End)
            .v_align(Align::Center)
            .right_margin(0.12)
            .left_margin(0.1)
            .v_margins(0.1)
            .build(&(name.to_owned() + " Outline"));

        let charging = Icon::builder()
            .icon('󱐋')
            .fg(self.charging_color)
            .bg(color::CLEAR)
            .h_align(Align::End)
            .v_align(Align::Center)
            .right_margin(0.02)
            .build(&(name.to_owned() + " Charging"));

        let progress = Progress::builder()
            .v_margins(0.4)
            .left_margin(0.17)
            .right_margin(0.2)
            .starting_bound(0.0)
            .ending_bound(1.0)
            .fill_direction(Direction::East)
            .filled_color(self.normal_color)
            .unfilled_color(color::CLEAR)
            .bg(color::CLEAR)
            .build(&(name.to_owned() + " Progress"));

        let battery_path = self
            .battery_path
            .clone()
            .unwrap_or_else(|| DEFAULT_BATTERY_PATH.into());
        assert!(battery_path.is_absolute());
        let battery_path = std::fs::canonicalize(battery_path.clone()).unwrap_or(battery_path);

        // should error if the path doesn't exist
        _ = std::fs::read_dir(battery_path.clone())?;

        Ok(Battery {
            name: name.into(),
            battery_path,
            desired_height,
            h_align: self.h_align,
            v_align: self.v_align,
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
