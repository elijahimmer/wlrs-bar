mod worker;
use worker::{work, ManagerMsg, WorkerMsg};

use crate::draw::prelude::*;
use crate::log::*;
use crate::widget::{ClickType, Widget};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use anyhow::Result;
use rusttype::Font;
use std::marker::PhantomData;

pub struct Volume {
    lc: LC,
    area: Rect,

    bg: Color,

    text: TextBox,
    progress: Progress,

    worker_handle: JoinHandle<Result<()>>,
    worker_send: Sender<ManagerMsg>,
    worker_recv: Receiver<WorkerMsg>,
}

impl Volume {
    pub fn builder() -> VolumeBuilder<NeedsFont> {
        VolumeBuilder::<NeedsFont>::new()
    }
}

impl Widget for Volume {
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
        true
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if ctx.full_redraw {
            trace!(self.lc, "| draw :: full redraw");

            self.area.draw(self.bg, ctx);
        }

        #[cfg(feature = "volume-outlines")]
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
pub struct VolumeBuilder<T> {
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

impl<T> VolumeBuilder<T> {
    pub fn new() -> VolumeBuilder<NeedsFont> {
        Default::default()
    }

    crate::builder_fields! {
        u32, desired_height;
        f32, show_threshold;
        Align, v_align h_align;
        Color, fg bg bar_filled;
    }

    pub fn font(self, font: Font<'static>) -> VolumeBuilder<HasFont> {
        VolumeBuilder {
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

impl VolumeBuilder<HasFont> {
    pub fn build(&self, lc: LC) -> Result<Volume> {
        let height = self.desired_height.unwrap_or(u32::MAX);
        info!(lc, "Initializing with height: {height}");
        let font = self.font.clone().unwrap();

        let text = TextBox::builder()
            .font(font)
            .v_align(self.v_align)
            .h_align(self.h_align)
            .right_margin(self.desired_height.unwrap_or(0) / 5)
            .fg(self.fg)
            .bg(color::CLEAR)
            .h_align(Align::CenterAt(0.55))
            .text("")
            .desired_text_height(self.desired_height.map(|s| s * 20 / 23).unwrap_or(u32::MAX))
            .build(lc.child("Text"));

        let mut progress = Progress::builder()
            .unfilled_color(color::CLEAR)
            .filled_color(self.bar_filled)
            .bg(self.bg)
            .starting_bound(0.0)
            .ending_bound(100.0)
            .desired_height(height)
            .build(lc.child("Progress"));

        progress.set_progress(0.0);

        let (send_to_worker, recv_from_main) = channel::<ManagerMsg>();
        let (send_to_main, recv_from_worker) = channel::<WorkerMsg>();

        let wkr_lc = lc
            .child("Worker Thread")
            .with_log(cfg!(feature = "volume-worker-logs"));
        let worker_handle = std::thread::Builder::new()
            .name(lc.name.to_string())
            .stack_size(32 * 1024)
            .spawn(move || work(wkr_lc, recv_from_main, send_to_main))?;

        Ok(Volume {
            lc,
            text,
            progress,
            bg: self.bg,
            area: Default::default(),

            worker_handle,
            worker_send: send_to_worker,
            worker_recv: recv_from_worker,
        })
    }
}
