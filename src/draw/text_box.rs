use crate::color::{self, Color};
use crate::draw::*;
use crate::widget::Widget;

use anyhow::Result;
use rusttype::{Font, PositionedGlyph};

#[derive(Clone)]
pub struct TextBox<'glyphs> {
    font: &'glyphs Font<'glyphs>,

    text: String,
    text_first_diff: u32,
    name: String,
    pub fg: Color,
    pub bg: Color,

    glyphs_size: Point<f32>,
    glyphs: Option<Vec<PositionedGlyph<'glyphs>>>,
    scale: Scale,

    area: Rect<f32>,
    desired_text_height: f32,

    redraw: bool,
    rerender_text: bool,
}

fn render_glyphs<'a>(
    font: &'a Font<'a>,
    text: &str,
    scale: Scale,
) -> (Vec<PositionedGlyph<'a>>, f32) {
    let v_metrics = font.v_metrics(scale);
    let offset = Point::new(0.0, v_metrics.ascent);

    let glyphs: Vec<_> = font.layout(text, scale, offset.into()).collect();
    let width = glyphs.last().map_or_else(
        || 0.0,
        |g| g.position().x + g.unpositioned().h_metrics().advance_width,
    );

    (glyphs, width)
}

impl TextBox<'_> {
    pub fn set_text(&mut self, new_text: String) {
        let new_text = new_text.trim();
        if new_text.is_empty() {
            self.glyphs_size = Point::new(0.0, 0.0);
        }
        self.rerender_text = self.text != new_text;
        self.text = new_text.to_string();
    }
}

impl Widget for TextBox<'_> {
    fn name(&self) -> &String {
        &self.name
    }
    fn area(&self) -> Rect<f32> {
        self.area
    }

    fn desired_height(&self) -> f32 {
        self.desired_text_height
    }

    fn desired_width(&self, height: f32) -> f32 {
        if self.text.is_empty() || height <= 0.0 {
            log::debug!("'{}', desired_width, returning 0", self.name);
            return 0.0;
        }

        let scale = Scale::uniform(height.clamp(0.0, self.desired_text_height));
        let (_glyphs, width) = render_glyphs(self.font, &self.text, scale);

        width
    }

    fn resize(&mut self, rect: Rect<f32>) {
        self.area = rect;
        self.redraw = true;
        self.rerender_text = false;

        let width_max = rect.width();
        let height_used = rect.height().clamp(0.0, self.desired_text_height);

        if width_max <= 0.0 || height_used == 0.0 {
            self.glyphs_size = Point::new(0.0, 0.0);
            self.glyphs = None;
            return;
        }

        self.scale = Scale::uniform(height_used);

        let (glyphs, width_used) = render_glyphs(self.font, &self.text, self.scale);

        if width_used <= width_max {
            log::trace!("'{}', resize, using desired scale", self.name);
            self.glyphs_size = Point::new(width_used, height_used);
            self.glyphs = Some(glyphs);
        } else {
            let ratio = width_max / width_used;
            log::trace!(
                "'{}', resize, area too small scaling to {ratio} times",
                self.name
            );
            debug_assert!((0.0..=1.0).contains(&ratio));

            self.scale = Scale::uniform((width_max * ratio).floor());
            log::trace!("'{}' calculated scale: {:?}", self.name, self.scale);
            let (new_glyphs, width_used_new) = render_glyphs(self.font, &self.text, self.scale);

            debug_assert!(width_used_new <= width_max);

            self.glyphs_size = Point::new(width_used_new, height_used);
            self.glyphs = Some(new_glyphs);
        };
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        let area: Rect<u32> = self.area.into();
        if self.glyphs_size.x <= 0.0 || self.glyphs_size.y <= 0.0 {
            // glyphs are 0 width
            return Ok(());
        }
        if !ctx.full_redraw && !self.redraw && !self.rerender_text {
            return Ok(());
        }

        assert!(!self.text.is_empty());
        assert!(self.glyphs.is_some());
        assert!(self.area.size() >= self.glyphs_size);

        let area_used = area.place_at(self.glyphs_size.into(), Align::Center, Align::Center);

        if self.rerender_text {
            log::trace!("'{}', draw, re-rendering glyphs", self.name);
            let (glyphs, width) = render_glyphs(self.font, &self.text, self.scale);
            self.glyphs = Some(glyphs);
            self.glyphs_size = Point::new(width, area_used.height() as f32);
        } else {
            log::trace!("'{}', draw, redrawing fully", self.name);
            area.draw(self.bg, ctx);
        }

        let glyphs_now = self
            .glyphs
            .as_ref()
            .unwrap()
            .iter()
            .zip(self.text.chars().fuse());

        for (gly, ch) in glyphs_now {
            if let Some(bb) = gly.pixel_bounding_box() {
                let mut rect: Rect<u32> = bb.into();
                rect.min.y += area_used.min.y;
                rect.max.y += area_used.min.y;
                rect.min.x += area_used.min.x;
                rect.max.x += area_used.min.x;

                debug_assert!(area_used.contains_rect(rect));

                ctx.damage.push(rect);
                //rect.draw_outline(self.fg, ctx);
                gly.draw(|x, y, v| {
                    let color = self.bg.blend(self.fg, v);
                    let point = Point::new(rect.min.x + x, rect.min.y + y);

                    debug_assert!(area_used.contains(point));
                    ctx.put(point, color);
                })
            }
        }

        if self.rerender_text {
            ctx.damage.push(area_used);
        }

        self.redraw = false;

        Ok(())
    }
}

impl<'a> TextBox<'a> {
    pub fn builder() -> TextBoxBuilder<'a> {
        TextBoxBuilder::new()
    }
}

#[derive(Clone)]
pub struct TextBoxBuilder<'glyphs> {
    font: &'glyphs Font<'glyphs>,
    text: String,
    fg: Color,
    bg: Color,
    desired_text_height: f32,
}

impl<'glyphs> TextBoxBuilder<'glyphs> {
    pub fn new() -> TextBoxBuilder<'glyphs> {
        Self {
            font: &FONT,
            text: String::new(),
            fg: *color::LOVE,
            bg: *color::SURFACE,
            desired_text_height: f32::INFINITY,
        }
    }

    pub fn font(mut self, font: &'glyphs Font<'glyphs>) -> Self {
        self.font = font;
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = text.trim().to_string();
        self
    }

    pub fn fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    pub fn desired_height(mut self, des: f32) -> Self {
        self.desired_text_height = des;
        self
    }

    pub fn build(self, name: String) -> TextBox<'glyphs> {
        TextBox {
            font: self.font,
            text: self.text,
            fg: self.fg,
            bg: self.bg,
            desired_text_height: self.desired_text_height,
            name,

            scale: Scale::uniform(0.0),
            area: Default::default(),
            glyphs: None,
            glyphs_size: Default::default(),
            redraw: false,
            rerender_text: false,
        }
    }
}

impl<'glyphs> Default for TextBoxBuilder<'glyphs> {
    fn default() -> Self {
        Self::new()
    }
}
