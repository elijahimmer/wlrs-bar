pub mod utils;

use crate::color;
use crate::draw::*;
use crate::widget::*;
use utils::*;

use anyhow::Result;
use std::io::Read;
use std::os::unix::net::UnixStream;

pub struct Workspaces<'a> {
    name: Box<str>,
    desired_height: f32,
    area: Rect<f32>,
    h_align: Align,
    v_align: Align,

    socket: UnixStream,

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

        Ok(Workspaces {
            name,
            h_align,
            v_align,
            desired_height,
            workspace_builder,
            socket: open_hypr_socket(HyprSocket::Event)?,

            active_workspace: Default::default(),
            workspaces: Default::default(),
            area: Default::default(),
        })
    }

    fn update_workspaces(&mut self) {
        //self.socket.read();
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
