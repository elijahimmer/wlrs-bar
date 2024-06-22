pub mod utils;
pub mod worker;

use crate::draw::prelude::*;
use crate::widget::*;
use utils::WorkspaceID;
use worker::{work, ManagerMsg, WorkerMsg};

use anyhow::Result;
use rusttype::Font;
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

pub struct Workspaces<'a> {
    name: Box<str>,
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

    workspace_builder: TextBoxBuilder<'a>,
    workspaces: Vec<(WorkspaceID, TextBox<'a>)>,
    active_workspace: WorkspaceID,
}

impl Workspaces<'_> {
    pub fn builder<'a>() -> WorkspacesBuilder<'a> {
        Default::default()
    }

    fn update_workspaces(&mut self) -> Result<()> {
        if self.worker_handle.is_none()
            || self.worker_handle.as_ref().is_some_and(|h| h.is_finished())
        {
            match self.worker_handle.take().map(|w| w.join()).transpose() {
                Ok(_) => log::warn!("'{}', workspaces worker returned too soon", self.name),
                Err(err) => log::error!(
                    "'{}', workspaces worker thread panicked. error={err:?}",
                    self.name
                ),
            }

            let (worker_send, other_recv) = mpsc::channel::<ManagerMsg>();
            let (other_send, worker_recv) = mpsc::channel::<WorkerMsg>();
            self.worker_send = worker_send;
            self.worker_recv = worker_recv;

            let wrk_name = self.name.to_owned();
            self.worker_handle = Some(std::thread::spawn(move || {
                work(&wrk_name, other_recv, other_send)
            }));
        }

        self.worker_recv.try_iter().for_each(|m| {
            #[cfg(feature = "workspaces-log")]
            log::trace!("'{}' | update_workspaces :: got msg: '{m:?}'", self.name);
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
                    } else {
                        #[cfg(feature = "workspaces-log")]
                        log::warn!(
                            "'{}' | update_workspaces :: previous active workspace doesn't exist",
                            self.name
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
                    } else {
                        #[cfg(feature = "workspaces-log")]
                        log::warn!(
                            "'{}' | update_workspaces :: new active workspace doesn't exist",
                            self.name
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
                            .build(&format!("{} {wk_name}", self.name));
                        self.workspaces.insert(idx, (id, wk));
                    } else {
                        #[cfg(feature = "workspaces-log")]
                        log::warn!(
                            "'{}' | update_workspaces :: workspace created that already exists",
                            self.name
                        );
                    }

                    self.redraw |= RedrawState::ReplaceNormal;
                }
                WorkerMsg::WorkspaceDestroy(id) => {
                    if let Ok(idx) = self.workspaces.binary_search_by_key(&id, |w| w.0) {
                        self.workspaces.remove(idx);
                    } else {
                        #[cfg(feature = "workspaces-log")]
                        log::debug!(
                            "'{}' | update_workspaces :: workspace destroyed that doesn't exists",
                            self.name
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
        let Point { y: height, .. } = self.area.size();

        let mut wk_area = self.area;
        wk_area.max.x = wk_area.min.x + height;
        for (idx, (_id, ref mut w)) in self.workspaces.iter_mut().enumerate() {
            log::trace!(
                "'{}' | resize :: wk_area: {wk_area}, size: {}",
                self.name,
                wk_area.size()
            );

            assert!(self.area.contains_rect(wk_area));
            assert!(wk_area.size() == Point::new(height, height));

            let old_area = w.area();
            w.resize(wk_area);

            if let Some((_hover_idx, hover_point)) =
                self.last_hover.filter(|(_hover_idx, hover_point)| {
                    old_area.contains(*hover_point) && !wk_area.contains(*hover_point)
                })
            {
                w.motion_leave(hover_point).unwrap();
            } else if let Some((_hover_idx, hover_point)) = self
                .last_hover
                .filter(|(_hover_idx, hover_point)| wk_area.contains(*hover_point))
            {
                #[cfg(feature = "workspaces-log")]
                log::trace!(
                    "'{}' | resize :: widget '{}' is new hover target",
                    self.name,
                    w.name()
                );
                w.motion(hover_point).unwrap();
                self.last_hover = Some((idx, hover_point));
            }

            wk_area = wk_area.x_shift(height.try_into().unwrap());
        }
    }
}

impl Drop for Workspaces<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.worker_send.send(worker::ManagerMsg::Close) {
            log::error!(
                "'{}', failed to send the thread a message. error={err}",
                self.name
            )
        }

        if let Err(err) = self.worker_handle.take().map(|w| w.join()).transpose() {
            log::error!(
                "'{}', workspaces worker thread panicked. error={err:?}",
                self.name
            )
        }
    }
}

impl Widget for Workspaces<'_> {
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
        self.workspaces
            .iter()
            .map(|(_idx, w)| w.desired_width(height))
            .sum::<u32>()
            .max(height * 12) // 12 workspaces worth- that should be enough
    }
    fn resize(&mut self, area: Rect) {
        self.area = area;
        self.redraw |= RedrawState::FillAfter;

        self.replace_widgets();
    }

    fn should_redraw(&mut self) -> bool {
        if let Err(err) = self.update_workspaces() {
            log::warn!(
                "'{}' | should_redraw :: failed to update workspaces. error={err}",
                self.name
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
                Rect::new(Point::new(w.area().max.x, self.area.min.y), self.area.max)
            });
            area_to_fill.draw(self.bg, ctx);
            ctx.damage.push(area_to_fill);
        } else {
            assert!(self.redraw.contains(RedrawState::Normal));
        }

        #[cfg(feature = "workspaces-log")]
        log::trace!("'{}' | draw :: Redraw State: {:?}", self.name, self.redraw);

        self.redraw = RedrawState::empty();

        self.workspaces.iter_mut().for_each(|(_idx, w)| {
            assert!(self.area.contains_rect(w.area()));
            if let Err(err) = w.draw(ctx) {
                log::warn!(
                    "'{}', widget '{}' failed to draw. error={err}",
                    self.name,
                    w.name()
                );
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
            #[cfg(feature = "workspaces-log")]
            log::debug!("'{}' | click :: clicked: {}", self.name, _w.name());
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
pub struct WorkspacesBuilder<'font> {
    font: Option<Font<'font>>,
    desired_height: u32,
    h_align: Align,
    v_align: Align,
    fg: Color,
    bg: Color,
    active_fg: Color,
    active_bg: Color,
    hover_fg: Color,
    hover_bg: Color,
}

impl<'font> WorkspacesBuilder<'font> {
    pub fn new() -> Self {
        Default::default()
    }

    crate::builder_fields! {
        Font<'font>, font;
        u32, desired_height;
        Align, v_align h_align;
        Color, fg bg active_fg active_bg hover_fg hover_bg;
    }

    pub fn build(&self, name: &str) -> Result<Workspaces<'font>> {
        log::info!(
            "'{name}' :: Initializing with height: {}",
            self.desired_height
        );

        let font = self
            .font
            .clone()
            .unwrap_or_else(|| panic!("'{}' A font must be provided.", name));

        let workspace_builder = TextBox::builder()
            .font(font)
            .fg(self.fg)
            .bg(self.bg)
            .hover_fg(self.hover_fg)
            .hover_bg(self.hover_bg)
            .h_align(Align::Center)
            .v_align(Align::Center)
            .desired_text_height(self.desired_height * 20 / 23);

        let (worker_send, other_recv) = mpsc::channel::<ManagerMsg>();
        let (other_send, worker_recv) = mpsc::channel::<WorkerMsg>();

        let wrk_name = name.to_owned();
        let worker_handle = Some(
            std::thread::Builder::new()
                .name(name.to_owned())
                .stack_size(32 * 1024)
                .spawn(move || work(&wrk_name, other_recv, other_send))?,
        );

        Ok(Workspaces {
            workspace_builder,
            worker_handle,
            worker_send,
            worker_recv,

            name: name.into(),
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
