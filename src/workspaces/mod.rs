pub mod utils;
pub mod worker;

use crate::draw::prelude::*;
use crate::log::*;
use crate::widget::*;
use utils::WorkspaceID;
use worker::{work, ManagerMsg, WorkerMsg};

use anyhow::Result;
use rusttype::Font;
use std::marker::PhantomData;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct RedrawState: u8 {
        /// just tell textboxes to redraw
        const Normal = 0b001;
        /// should re-place the widgets
        const Replace = 0b010;
        /// fill in space after the text-boxes
        const FillAfter = 0b100;

        const ReplaceNormal = Self::Replace.bits() | Self::Normal.bits();
        const ReplaceFill = Self::Replace.bits() | Self::FillAfter.bits();
    }
}

pub struct Workspaces {
    lc: LC,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,
    fg: Color,
    bg: Color,
    active_fg: Color,
    active_bg: Color,
    redraw: RedrawState,

    last_hover: Option<(usize, Point)>,

    worker_handle: Option<JoinHandle<Result<()>>>,
    worker_send: Sender<ManagerMsg>,
    worker_recv: Receiver<WorkerMsg>,

    workspace_builder: TextBoxBuilder<HasFont>,
    workspaces: Vec<(WorkspaceID, TextBox)>,
    active_workspace: WorkspaceID,
}

impl Workspaces {
    pub fn builder() -> WorkspacesBuilder<NeedsFont> {
        Default::default()
    }

    fn update_workspaces(&mut self) -> Result<()> {
        if self.worker_handle.is_none()
            || self.worker_handle.as_ref().is_some_and(|h| h.is_finished())
        {
            match self.worker_handle.take().map(|w| w.join()).transpose() {
                Ok(_) => warn!("'{}', workspaces worker returned too soon", self.lc),
                Err(err) => error!(
                    "'{}', workspaces worker thread panicked. error={err:?}",
                    self.lc
                ),
            }

            let (worker_send, other_recv) = mpsc::channel::<ManagerMsg>();
            let (other_send, worker_recv) = mpsc::channel::<WorkerMsg>();
            self.worker_send = worker_send;
            self.worker_recv = worker_recv;

            let wkr_lc = self.lc.child("Worker Thread");
            self.worker_handle = Some(
                std::thread::Builder::new()
                    .name(self.lc.name.to_string())
                    .stack_size(32 * 1024)
                    .spawn(move || work(wkr_lc, other_recv, other_send))?,
            );
        }

        self.worker_recv.try_iter().for_each(|m| {
            if self.lc.should_log {
                trace!("'{}' | update_workspaces :: got msg: '{m:?}'", self.lc);
            }
            match m {
                WorkerMsg::WorkspaceReset => {
                    self.workspaces.clear();
                    self.redraw |= RedrawState::Normal;
                }
                WorkerMsg::WorkspaceSetActive(id) => {
                    if let Some((_id, w)) = self
                        .workspaces
                        .binary_search_by_key(&self.active_workspace, |w| w.0)
                        .ok()
                        .and_then(|idx| self.workspaces.get_mut(idx))
                    {
                        w.set_fg(self.fg);
                        w.set_bg(self.bg);
                    } else if self.lc.should_log {
                        warn!(
                            "'{}' | update_workspaces :: previous active workspace doesn't exist",
                            self.lc
                        );
                    }

                    self.active_workspace = id;
                    if let Some((_id, w)) = self
                        .workspaces
                        .binary_search_by_key(&id, |w| w.0)
                        .ok()
                        .and_then(|idx| self.workspaces.get_mut(idx))
                    {
                        w.set_fg(self.active_fg);
                        w.set_bg(self.active_bg);
                    } else if self.lc.should_log {
                        warn!(
                            "'{}' | update_workspaces :: new active workspace doesn't exist",
                            self.lc
                        );
                    }
                    self.redraw |= RedrawState::Normal;
                }
                WorkerMsg::WorkspaceCreate(id) => {
                    if let Err(idx) = self.workspaces.binary_search_by_key(&id, |w| w.0) {
                        let wk_name = utils::map_workspace_id(id);

                        let mut builder = self.workspace_builder.clone();

                        if id == self.active_workspace {
                            builder = builder.fg(self.active_fg).bg(self.active_bg);
                        }

                        let wk = builder
                            .text(wk_name.as_str())
                            .build(self.lc.child(&wk_name));
                        self.workspaces.insert(idx, (id, wk));
                    } else if self.lc.should_log {
                        warn!(
                            "'{}' | update_workspaces :: workspace created that already exists",
                            self.lc
                        );
                    }

                    self.redraw |= RedrawState::ReplaceNormal;
                }
                WorkerMsg::WorkspaceDestroy(id) => {
                    if let Ok(idx) = self.workspaces.binary_search_by_key(&id, |w| w.0) {
                        self.workspaces.remove(idx);
                    } else if self.lc.should_log {
                        debug!(
                            "{} | update_workspaces :: workspace destroyed that doesn't exists",
                            self.lc
                        );
                    }
                    self.redraw |= RedrawState::ReplaceFill;
                }
            }
        });

        Ok(())
    }

    fn replace_widgets(&mut self) {
        self.redraw -= RedrawState::Replace;

        let mut workspaces = self
            .workspaces
            .iter_mut()
            .map(|w| &mut w.1 as &mut dyn Widget)
            .collect::<Vec<_>>();

        crate::widget::stack_widgets_right(&self.lc, &mut workspaces, self.area);
    }
}

impl Drop for Workspaces {
    fn drop(&mut self) {
        if let Err(err) = self.worker_send.send(worker::ManagerMsg::Close) {
            error!(
                "'{}', failed to send the thread a message. error={err}",
                self.lc
            )
        }

        if let Err(err) = self.worker_handle.take().map(|w| w.join()).transpose() {
            error!(
                "'{}', workspaces worker thread panicked. error={err:?}",
                self.lc
            )
        }
    }
}

impl Widget for Workspaces {
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
        self.workspaces
            .iter()
            .map(|(_idx, w)| w.desired_width(height))
            .sum::<u32>()
            .max(height * 10) // 10 workspaces worth- that should be enough
    }
    fn resize(&mut self, area: Rect) {
        self.area = area;
        self.redraw |= RedrawState::FillAfter;

        self.replace_widgets();
    }

    fn should_redraw(&mut self) -> bool {
        if let Err(err) = self.update_workspaces() {
            warn!(
                "'{}' | should_redraw :: failed to update workspaces. error={err}",
                self.lc
            );
        }

        !self.redraw.is_empty()
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if self.redraw.contains(RedrawState::Replace) {
            self.replace_widgets();
        }

        if ctx.full_redraw {
            self.area.draw(self.bg, ctx);
        } else if self.redraw.is_empty() {
            return Ok(());
        } else if self.redraw.contains(RedrawState::FillAfter) {
            let area_to_fill = self.workspaces.last().map_or(self.area, |(_id, w)| {
                Rect::new(
                    Point {
                        x: w.area().max.x,
                        y: self.area.min.y,
                    },
                    self.area.max,
                )
            });
            area_to_fill.draw(self.bg, ctx);
            ctx.damage.push(area_to_fill);
        } else {
            assert!(self.redraw.contains(RedrawState::Normal));
        }

        if self.lc.should_log {
            trace!("'{}' | draw :: Redraw State: {:?}", self.lc, self.redraw);
        }

        self.redraw = RedrawState::empty();

        self.workspaces.iter_mut().for_each(|(_idx, w)| {
            assert!(self.area.contains_rect(w.area()));
            if w.should_redraw() {
                if let Err(err) = w.draw(ctx) {
                    warn!(
                        "'{}', widget '{}' failed to draw. error={err}",
                        self.lc,
                        w.lc()
                    );
                }
            }
            #[cfg(feature = "workspaces-outlines")]
            w.area().draw_outline(crate::draw::color::IRIS, ctx);
        });

        Ok(())
    }

    fn click(&mut self, button: ClickType, point: Point) -> Result<()> {
        if button != ClickType::LeftClick {
            return Ok(());
        }

        if let Some((id, _w)) = self.workspaces.iter().find(|w| w.1.area().contains(point)) {
            #[cfg(feature = "workspaces-logs")]
            debug!("'{}' | click :: clicked: {}", self.name, _w.name());
            let _ = utils::send_hypr_command(utils::Command::MoveToWorkspace(*id))?;
        }

        Ok(())
    }

    fn motion(&mut self, point: Point) -> Result<()> {
        let moved_in_idx = self
            .workspaces
            .iter_mut()
            .enumerate()
            .find(|(_idx, (_id, w))| w.area().contains(point))
            .map(|(idx, (_id, w))| {
                w.motion(point).unwrap();

                (idx, point)
            });

        if self.last_hover.unzip().0 != moved_in_idx.unzip().0 {
            if let Some((_id, w)) = self
                .last_hover
                .and_then(|(idx, _area)| self.workspaces.get_mut(idx))
            {
                w.motion_leave(point).unwrap();
            }
        }

        self.last_hover = moved_in_idx;
        self.redraw |= RedrawState::Normal;

        Ok(())
    }
    fn motion_leave(&mut self, point: Point) -> Result<()> {
        if let Some((_id, w)) = self
            .last_hover
            .take()
            .and_then(|(idx, _area)| self.workspaces.get_mut(idx))
        {
            w.motion_leave(point).unwrap();
        }
        self.redraw |= RedrawState::Normal;

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorkspacesBuilder<T> {
    font: Option<Font<'static>>,
    desired_height: u32,
    h_align: Align,
    v_align: Align,
    fg: Color,
    bg: Color,
    active_fg: Color,
    active_bg: Color,
    hover_fg: Color,
    hover_bg: Color,

    _state: PhantomData<T>,
}

impl<T> WorkspacesBuilder<T> {
    pub fn new() -> WorkspacesBuilder<NeedsFont> {
        Default::default()
    }

    crate::builder_fields! {
        u32, desired_height;
        Align, v_align h_align;
        Color, fg bg active_fg active_bg hover_fg hover_bg;
    }

    pub fn font(self, font: Font<'static>) -> WorkspacesBuilder<HasFont> {
        WorkspacesBuilder {
            _state: PhantomData,
            font: Some(font),

            h_align: self.h_align,
            v_align: self.v_align,
            desired_height: self.desired_height,
            fg: self.fg,
            bg: self.bg,
            active_fg: self.active_fg,
            active_bg: self.active_bg,
            hover_fg: self.hover_fg,
            hover_bg: self.hover_bg,
        }
    }
}

impl WorkspacesBuilder<HasFont> {
    pub fn build(&self, lc: LC) -> Result<Workspaces> {
        info!("{lc} :: Initializing with height: {}", self.desired_height);

        let font = self.font.clone().unwrap();

        let workspace_builder = TextBox::builder()
            .font(font)
            .fg(self.fg)
            .bg(self.bg)
            .hover_fg(self.hover_fg)
            .hover_bg(self.hover_bg)
            .h_align(Align::Center)
            .v_align(Align::Center)
            .desired_text_height(self.desired_height * 20 / 23)
            .desired_width(self.desired_height);

        let (worker_send, other_recv) = mpsc::channel::<ManagerMsg>();
        let (other_send, worker_recv) = mpsc::channel::<WorkerMsg>();

        let wkr_lc = lc.child("Worker Thread");
        let worker_handle = Some(
            std::thread::Builder::new()
                .name(lc.name.to_string())
                .stack_size(32 * 1024)
                .spawn(move || work(wkr_lc, other_recv, other_send))?,
        );

        Ok(Workspaces {
            workspace_builder,
            worker_handle,
            worker_send,
            worker_recv,
            lc,

            h_align: self.h_align,
            v_align: self.v_align,
            desired_height: self.desired_height,
            fg: self.fg,
            bg: self.bg,
            active_fg: self.active_fg,
            active_bg: self.active_bg,

            active_workspace: 1,
            last_hover: Default::default(),
            workspaces: Default::default(),
            area: Default::default(),
            redraw: RedrawState::empty(),
        })
    }
}
