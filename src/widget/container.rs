use super::place_widgets::*;
use super::*;
use crate::log::LC;

//use crate::draw::prelude::*;

pub struct Container {
    lc: LC,
    widgets: Vec<Box<dyn Widget>>,
    should_redraw: Vec<bool>,
    v_align: Align,
    h_align: Align,
    inner_h_align: Align,
    area: Rect,

    last_motion: Option<Point>,

    desired_height: Option<u32>,
    desired_width: Option<u32>,
}

impl Container {
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }
}

impl Widget for Container {
    fn lc(&self) -> &LC {
        &self.lc
    }

    fn h_align(&self) -> Align {
        self.h_align
    }

    fn v_align(&self) -> Align {
        self.v_align
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn desired_height(&self) -> u32 {
        self.desired_height.unwrap_or_else(|| {
            self.widgets
                .iter()
                .map(|w| w.desired_height())
                .max()
                .unwrap_or(0)
        })
    }

    fn desired_width(&self, height: u32) -> u32 {
        self.desired_width
            .unwrap_or_else(|| self.widgets.iter().map(|w| w.desired_width(height)).sum())
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
        match self.inner_h_align {
            Align::Center => center_widgets(&self.lc, &mut self.widgets, area),
            Align::End => stack_widgets_left(&self.lc, &mut self.widgets, area),
            Align::Start => stack_widgets_right(&self.lc, &mut self.widgets, area),
            _ => todo!(),
        }
    }

    fn should_redraw(&mut self) -> bool {
        self.should_redraw = self
            .widgets
            .iter_mut()
            .map(|w| w.should_redraw())
            .collect::<Vec<_>>(); // make sure they are all asked to resize

        self.should_redraw.iter().any(|b| *b)
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        for (w, should) in self.widgets.iter_mut().zip(self.should_redraw.drain(..)) {
            if should {
                //log::info!("'{}' | draw :: drawing: {}", self.lc, w.lc().name);
                w.draw(ctx)?;
            }
        }

        Ok(())
    }

    fn motion(&mut self, point: Point) -> Result<()> {
        assert!(self.area.contains(point));
        self.last_motion.take().map(|p| {
            self.widgets
                .iter_mut()
                .find(|w| w.area().contains(p))
                .map(|w| w.motion_leave(point))
        });

        self.widgets
            .iter_mut()
            .find(|w| w.area().contains(point))
            .map(|w| w.motion(point));

        self.last_motion = Some(point);

        Ok(())
    }

    fn motion_leave(&mut self, point: Point) -> Result<()> {
        self.last_motion.take().map(|p| {
            self.widgets
                .iter_mut()
                .find(|w| w.area().contains(p))
                .map(|w| w.motion_leave(point))
        });

        Ok(())
    }

    fn click(&mut self, event: ClickType, point: Point) -> Result<()> {
        assert!(self.area.contains(point));
        self.widgets
            .iter_mut()
            .find(|w| w.area().contains(point))
            .map(|w| w.click(event, point));

        Ok(())
    }
}

#[derive(Default)]
pub struct ContainerBuilder {
    widgets: Vec<Box<dyn Widget>>,
    v_align: Align,
    h_align: Align,
    inner_h_align: Align,

    desired_height: Option<u32>,
    desired_width: Option<u32>,
}

impl ContainerBuilder {
    pub fn new() -> ContainerBuilder {
        Default::default()
    }

    crate::builder_fields! {
        Align, v_align h_align inner_h_align;
        u32, desired_height desired_width;
    }

    pub fn add(&mut self, widget: Box<dyn Widget>) -> &mut Self {
        self.widgets.push(widget);
        self
    }

    pub fn build(self, lc: LC) -> Container {
        Container {
            lc,
            should_redraw: Vec::with_capacity(self.widgets.len()),
            widgets: self.widgets,
            v_align: self.v_align,
            h_align: self.h_align,
            inner_h_align: self.inner_h_align,

            desired_width: self.desired_width,
            desired_height: self.desired_height,

            area: Default::default(),
            last_motion: Default::default(),
        }
    }
}
