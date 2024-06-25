use crate::draw::prelude::*;
use crate::log::*;
use crate::widget::{ClickType, Widget};

use anyhow::{bail, Result};
use chrono::{DateTime, TimeDelta, Utc};
use rusttype::Font;
use std::marker::PhantomData;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

bitflags::bitflags! {
    #[derive(Clone, Default, Debug)]
    pub struct RedrawState: u8 {
        const ShouldBeShown = 1;
        const CurrentlyShown = 1 << 1;
        const ProgressiveRedraw = 1 << 2;

        const ShownAsItShouldBe = Self::ShouldBeShown.bits() | Self::CurrentlyShown.bits();
    }
}

pub struct Cpu {
    lc: LC,
    cpu_tracker: System,
    cpu_refresh: CpuRefreshKind,
    show_threshold: f32,
    last_refreshed: DateTime<Utc>,
    refresh_interval: TimeDelta,
    redraw: RedrawState,
    area: Rect,

    bg: Color,

    text: TextBox,
    progress: Progress,
}

impl Cpu {
    pub fn builder() -> CpuBuilder<NeedsFont> {
        CpuBuilder::<NeedsFont>::new()
    }
}

impl Widget for Cpu {
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
        height
    }
    fn resize(&mut self, area: Rect) {
        self.area = area;
        self.text.resize(area);
        self.progress.resize(area);
    }
    fn should_redraw(&mut self) -> bool {
        let now = Utc::now();

        if now - self.last_refreshed <= self.refresh_interval {
            return false;
        }

        self.last_refreshed = now;
        self.cpu_tracker.refresh_cpu_specifics(self.cpu_refresh);

        let cpu_used = self
            .cpu_tracker
            .global_cpu_info()
            .cpu_usage()
            .clamp(0.0, 100.0);

        if cpu_used < self.show_threshold {
            if self.lc.should_log {
                debug!(
                    "{} | should_redraw :: shouldn't be shown {}",
                    self.lc, cpu_used
                );
            }
            self.redraw -= !RedrawState::CurrentlyShown;
            self.redraw.contains(RedrawState::CurrentlyShown)
        } else {
            if self.lc.should_log {
                debug!(
                    "{} | should_redraw :: should be shown {}",
                    self.lc, cpu_used
                );
            }
            self.redraw |= RedrawState::ShouldBeShown;

            self.progress.set_progress(cpu_used);
            // self.text.should_redraw(); // We don't need this right now
            if self.progress.should_redraw() {
                if self.lc.should_log {
                    info!("should update");
                }
                self.redraw |= RedrawState::ProgressiveRedraw;
            }
            self.redraw.contains(RedrawState::ProgressiveRedraw)
                || !self.redraw.contains(RedrawState::CurrentlyShown)
        }
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if ctx.full_redraw {
            if self.lc.should_log {
                trace!("{} | draw :: full redraw", self.lc);
            }

            self.area.draw(self.bg, ctx);
        }

        if self.redraw.contains(RedrawState::ShouldBeShown)
            && (ctx.full_redraw
                || self.redraw.contains(RedrawState::ProgressiveRedraw)
                || !self.redraw.contains(RedrawState::CurrentlyShown))
        {
            if self.lc.should_log {
                trace!("{} | draw :: showing widgets", self.lc);
            }
            self.redraw = RedrawState::ShownAsItShouldBe;
            self.progress.draw(ctx)?;
            self.text.draw(ctx)?;
        } else if self.redraw.contains(RedrawState::CurrentlyShown) {
            if self.lc.should_log {
                trace!("{} | draw :: not showing", self.lc);
            }
            self.redraw = RedrawState::empty();
            self.area.draw(self.bg, ctx);
        }

        #[cfg(feature = "cpu-outlines")]
        self.progress.area().draw_outline(color::LOVE, ctx);

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
pub struct CpuBuilder<T> {
    font: Option<Font<'static>>,
    desired_height: Option<u32>,
    h_align: Align,
    v_align: Align,
    fg: Color,
    bg: Color,
    bar_filled: Color,

    show_threshold: Option<f32>,

    _state: PhantomData<T>,
}

impl<T> CpuBuilder<T> {
    pub fn new() -> CpuBuilder<NeedsFont> {
        Default::default()
    }

    crate::builder_fields! {
        u32, desired_height;
        f32, show_threshold;
        Align, v_align h_align;
        Color, fg bg bar_filled;
    }

    pub fn font(self, font: Font<'static>) -> CpuBuilder<HasFont> {
        CpuBuilder {
            _state: PhantomData,
            font: Some(font),

            show_threshold: self.show_threshold,
            desired_height: self.desired_height,
            h_align: self.h_align,
            v_align: self.v_align,
            fg: self.fg,
            bg: self.bg,
            bar_filled: self.bar_filled,
        }
    }
}

impl CpuBuilder<HasFont> {
    pub fn build(&self, lc: LC) -> Result<Cpu> {
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            bail!("System not supported.");
        }
        let height = self.desired_height.unwrap_or(u32::MAX);
        info!("{lc} :: Initializing with height: {height}");
        let font = self.font.clone().unwrap();

        let text = TextBox::builder()
            .font(font)
            .v_align(self.v_align)
            .h_align(self.h_align)
            .right_margin(self.desired_height.unwrap_or(0) / 5)
            .fg(self.fg)
            .bg(color::CLEAR)
            .h_align(Align::CenterAt(0.575))
            .text("")
            .desired_text_height(self.desired_height.map(|s| s * 20 / 23).unwrap_or(u32::MAX))
            .build(lc.child("Text"));

        let cpu_refresh = CpuRefreshKind::new().with_cpu_usage().without_frequency();

        let refresh_kind = RefreshKind::new().with_cpu(cpu_refresh);

        let mut cpu_tracker = System::new_with_specifics(refresh_kind);
        cpu_tracker.refresh_cpu_specifics(cpu_refresh); // initial to get measurements correct

        let mut progress = Progress::builder()
            .unfilled_color(color::CLEAR)
            .filled_color(self.bar_filled)
            .bg(self.bg)
            .starting_bound(0.0)
            .ending_bound(100.0)
            .desired_height(height)
            .build(lc.child("Progress"));

        progress.set_progress(0.0);

        Ok(Cpu {
            lc,
            cpu_tracker,
            cpu_refresh,
            show_threshold: self.show_threshold.unwrap_or(75.0),
            text,
            progress,
            last_refreshed: Utc::now(),
            refresh_interval: TimeDelta::from_std(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).unwrap()
                * 2,
            bg: self.bg,
            redraw: Default::default(),
            area: Default::default(),
        })
    }
}
