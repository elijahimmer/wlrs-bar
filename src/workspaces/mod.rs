pub mod utils;
pub mod worker;

use crate::draw::*;
use crate::widget::*;
use utils::WorkspaceID;
use worker::{work, ManagerMsg, WorkerMsg};

use anyhow::Result;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

pub struct Workspaces<'a> {
    name: Box<str>,
    desired_height: u32,
    area: Rect,
    h_align: Align,
    v_align: Align,
    should_resize: bool,
    fg: Color,
    bg: Color,
    active_fg: Color,
    active_bg: Color,

    worker_handle: Option<JoinHandle<Result<()>>>,
    worker_send: Sender<ManagerMsg>,
    worker_recv: Receiver<WorkerMsg>,

    workspace_builder: TextBoxBuilder<'a>,
    workspaces: Vec<(WorkspaceID, TextBox<'a>)>,
    active_workspace: WorkspaceID,
}

impl Workspaces<'_> {
    pub fn new<'a>(
        name: &str,
        desired_height: u32,
        h_align: Align,
        v_align: Align,
    ) -> Result<Workspaces<'a>> {
        log::info!("'{name}' | new :: initializing with height: {desired_height}");

        let fg = color::ROSE;
        let bg = color::SURFACE;
        let active_fg = color::ROSE;
        let active_bg = color::PINE;

        let workspace_builder = TextBox::builder()
            .fg(fg)
            .bg(bg)
            .h_align(Align::Center)
            .v_align(Align::Center)
            .desired_text_height(desired_height * 20 / 23);

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
            name: name.into(),
            h_align,
            v_align,
            desired_height,
            workspace_builder,
            worker_handle,
            worker_send,
            worker_recv,
            fg,
            bg,
            active_fg,
            active_bg,

            active_workspace: 1,
            workspaces: Default::default(),
            area: Default::default(),
            should_resize: false,
        })
    }

    fn update_workspaces(&mut self) -> Result<()> {
        if self.worker_handle.is_none()
            || self.worker_handle.as_ref().is_some_and(|h| h.is_finished())
        {
            let _ = self
                .worker_handle
                .take()
                .map(|w| w.join())
                .transpose()
                .map_err(|err| {
                    log::error!(
                        "'{}', workspaces worker thread panicked. error={err:?}",
                        self.name
                    )
                })
                .map(|_| log::warn!("'{}', workspaces worker returned too soon", self.name));

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
            log::trace!("'{}', got msg: '{m:?}'", self.name);
            match m {
                WorkerMsg::WorkspaceReset => {
                    self.workspaces.clear();
                    self.should_resize = true;
                }
                WorkerMsg::WorkspaceSetActive(id) => {
                    let _ = self
                        .workspaces
                        .binary_search_by_key(&self.active_workspace, |w| w.0)
                        .map(|idx| {
                            self.workspaces.get_mut(idx).map(|(_id, w)| {
                                // workspace exists
                                w.set_fg(self.fg);
                                w.set_bg(self.bg);
                            })
                        });
                    self.active_workspace = id;
                    let _ = self
                        .workspaces
                        .binary_search_by_key(&id, |w| w.0)
                        .map(|idx| {
                            self.workspaces.get_mut(idx).map(|(_id, w)| {
                                // workspace exists
                                w.set_fg(self.active_fg);
                                w.set_bg(self.active_bg);
                            })
                        });
                }
                WorkerMsg::WorkspaceCreate(id) => {
                    let _ = self
                        .workspaces
                        .binary_search_by_key(&id, |w| w.0)
                        .map_err(|idx| {
                            let wk_name = utils::map_workspace_id(id);

                            let mut builder = self.workspace_builder.clone();

                            if id == self.active_workspace {
                                builder = builder.fg(self.active_fg).bg(self.active_bg);
                            }

                            let wk = builder
                                .text(wk_name.as_str())
                                .build(&format!("{} {wk_name}", self.name));
                            self.workspaces.insert(idx, (id, wk));
                        })
                        .map(|_idx| log::info!("'{}' update_workspace :: created already existing workspace id={id}", self.name));

                    self.should_resize = true;
                }
                WorkerMsg::WorkspaceDestroy(id) => {
                    let _ = self
                        .workspaces
                        .binary_search_by_key(&id, |w| w.0)
                        .map(|idx| self.workspaces.remove(idx))
                        .map_err(|_idx| {
                            log::warn!(
                                "'{}' update_workspaces :: destroyed non-existant workspace id={id}",
                                self.name
                            )
                        });
                    self.should_resize = true;
                }
            }
        });

        Ok(())
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
            .max(height * 12)
    }
    fn resize(&mut self, area: Rect) {
        let Point {
            x: _width,
            y: height,
        } = area.size();

        let mut wk_area = area;
        wk_area.max.x = wk_area.min.x + height;
        self.workspaces.iter_mut().for_each(|(_idx, w)| {
            log::trace!(
                "'{}' | resize :: wk_area: {wk_area}, size: {}",
                self.name,
                wk_area.size()
            );
            debug_assert!(area.contains_rect(wk_area));
            debug_assert!(wk_area.size() == Point::new(height, height));
            w.resize(wk_area);
            wk_area.min.x += height;
            wk_area.max.x += height;
        });
        self.area = area;
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.update_workspaces()?;
        let redraw = ctx.full_redraw;
        if self.should_resize || redraw {
            ctx.full_redraw = true;
            self.area.draw(self.bg, ctx);
            self.resize(self.area);
            self.should_resize = false;
        }

        self.workspaces.iter_mut().for_each(|(_idx, w)| {
            debug_assert!(self.area.contains_rect(w.area()));
            if let Err(err) = w.draw(ctx) {
                log::warn!(
                    "'{}', widget '{}' failed to draw. error={err}",
                    self.name,
                    w.name()
                );
            }
        });

        ctx.full_redraw = redraw;
        Ok(())
    }
}
