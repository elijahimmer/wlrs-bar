pub mod utils;
pub mod worker;

use crate::color;
use crate::draw::*;
use crate::widget::*;
use utils::*;

use anyhow::{anyhow, Result};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

pub struct Workspaces<'a> {
    name: Box<str>,
    desired_height: f32,
    area: Rect<f32>,
    h_align: Align,
    v_align: Align,

    worker_handle: Option<JoinHandle<Result<()>>>,
    worker_send: Sender<worker::ManagerMsg>,
    worker_recv: Receiver<worker::WorkerMsg>,

    workspace_builder: TextBoxBuilder<'a>,
    workspaces: Vec<(usize, TextBox<'a>)>,
    active_workspace: usize,
}

impl Workspaces<'_> {
    pub fn new<'a>(
        name: Box<str>,
        desired_height: f32,
        h_align: Align,
        v_align: Align,
    ) -> Result<Workspaces<'a>> {
        log::info!("'{name}' initializing with height: {desired_height}");

        let workspace_builder = TextBox::builder()
            .fg(*color::ROSE)
            .desired_width(desired_height)
            .desired_text_height(desired_height);

        let (worker_send, other_recv) = mpsc::channel::<worker::ManagerMsg>();
        let (other_send, worker_recv) = mpsc::channel::<worker::WorkerMsg>();

        let wrk_name = name.to_owned();
        let worker_handle = Some(std::thread::spawn(move || {
            worker::work(&wrk_name, other_recv, other_send)
        }));

        Ok(Workspaces {
            name,
            h_align,
            v_align,
            desired_height,
            workspace_builder,
            worker_handle,
            worker_send,
            worker_recv,

            workspaces: Default::default(),
            active_workspace: Default::default(),
            area: Default::default(),
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

            let (worker_send, other_recv) = mpsc::channel::<worker::ManagerMsg>();
            let (other_send, worker_recv) = mpsc::channel::<worker::WorkerMsg>();
            self.worker_send = worker_send;
            self.worker_recv = worker_recv;

            let wrk_name = self.name.to_owned();
            self.worker_handle = Some(std::thread::spawn(move || {
                worker::work(&wrk_name, other_recv, other_send)
            }));
        }

        Ok(())
    }
}

impl Drop for Workspaces<'_> {
    fn drop(&mut self) {
        let _ = self
            .worker_send
            .send(worker::ManagerMsg::Close)
            .map_err(|err| {
                log::error!(
                    "'{}', failed to send the thread a message. error={err}",
                    self.name
                )
            });
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
            });
    }
}

impl Widget for Workspaces<'_> {
    fn name(&self) -> &str {
        &self.name
    }
    fn area(&self) -> Rect<f32> {
        self.area
    }
    fn h_align(&self) -> Align {
        self.h_align
    }
    fn v_align(&self) -> Align {
        self.v_align
    }
    fn desired_height(&self) -> f32 {
        self.desired_height
    }
    fn desired_width(&self, height: f32) -> f32 {
        self.workspaces
            .iter()
            .map(|(_idx, w)| w.desired_width(height))
            .sum()
    }
    fn resize(&mut self, area: Rect<f32>) {
        center_widgets(
            self.workspaces
                .iter_mut()
                .map(|(_idx, w)| &mut (*w))
                .collect::<Vec<_>>()
                .as_mut_slice(),
            area,
        );
        self.area = area;
    }
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.update_workspaces()?;

        self.workspaces.iter_mut().for_each(|(_idx, w)| {
            if let Err(err) = w.draw(ctx) {
                log::warn!(
                    "'{}', widget '{}' failed to draw. error={err}",
                    self.name,
                    w.name()
                );
            }
        });
        Ok(())
    }
}
