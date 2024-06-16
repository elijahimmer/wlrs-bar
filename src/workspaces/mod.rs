pub mod utils;

use crate::color;
use crate::draw::*;
use crate::widget::*;

use anyhow::Result;

pub struct Workspaces<'a> {
    name: Box<str>,
    desired_height: f32,
    area: Rect<f32>,
    h_align: Align,
    v_align: Align,

    workspace_builder: TextBoxBuilder<'a>,
    workspaces: Vec<TextBox<'a>>,
}

impl Workspaces<'_> {
    pub fn new<'a>(
        name: Box<str>,
        desired_height: f32,
        h_align: Align,
        v_align: Align,
    ) -> Workspaces<'a> {
        log::info!("'{name}' initializing with height: {desired_height}");

        let workspace_builder = TextBox::builder()
            .fg(*color::ROSE)
            .desired_width(desired_height)
            .desired_text_height(desired_height);

        Workspaces {
            name,
            h_align,
            v_align,
            desired_height,
            workspace_builder,

            workspaces: Default::default(),
            area: Default::default(),
        }
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
            .fold(0.0, |acc, w| acc + w.desired_width(height))
    }
    fn resize(&mut self, area: Rect<f32>) {
        center_widgets(
            self.workspaces
                .iter_mut()
                .map(|w| &mut (*w))
                .collect::<Vec<_>>()
                .as_mut_slice(),
            area,
        );
        self.area = area;
    }
    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        self.workspaces.iter_mut().for_each(|w| {
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

const ALPHA_CHAR: u32 = 'Α' as u32 - 1;

pub fn map_workspace(workspace: u32) -> String {
    match workspace {
        i @ 1..=17 => match char::from_u32(ALPHA_CHAR + i) {
            Some(ch) => ch.to_string(),
            None => {
                log::warn!("Failed to map workspace to symbol: i={i}");
                format!("{}", i)
            }
        },
        // I needed to split this because there is a reserved character between rho and sigma.
        i @ 18..=24 => match char::from_u32((ALPHA_CHAR + 1) + i) {
            Some(ch) => ch.to_string(),
            None => {
                log::warn!("Failed to map workspace to symbol: i={i}");
                format!("{}", i)
            }
        },
        i => format!("{}", i),
    }
}
