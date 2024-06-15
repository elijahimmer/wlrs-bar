use crate::color::{self, Color};
use crate::draw::*;
use crate::widget::Widget;

use anyhow::Result;
use rusttype::{Font, PositionedGlyph};

pub struct TextBox<'glyphs> {
    font: &'glyphs Font<'glyphs>,

    pub text: String,
    pub name: String,
    pub fg: Color,
    pub bg: Color,

    text_drawn_last: Option<String>,

    glyphs: Option<Vec<PositionedGlyph<'glyphs>>>,
    glyphs_drawn_last: Option<Vec<PositionedGlyph<'glyphs>>>,
    scale: Scale,

    area: Rect<f32>,
    desired_text_height: f32,

    redraw: bool,
}

fn render_glyphs<'a>(
    font: &'a Font<'a>,
    text: &str,
    scale: Scale,
    x_offset: f32,
) -> (Vec<PositionedGlyph<'a>>, f32) {
    let v_metrics = font.v_metrics(scale);
    let offset = Point::new(x_offset, v_metrics.ascent);

    let glyphs: Vec<_> = font.layout(text, scale, offset.into()).collect();
    let width = glyphs.last().map_or_else(
        || 0.0,
        |g| g.position().x + g.unpositioned().h_metrics().advance_width,
    );

    (glyphs, width - x_offset)
}

impl TextBox<'_> {
    pub fn set_text(&mut self, new_text: String) {
        self.text = new_text;
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
            return 0.0;
        }

        let scale = Scale::uniform(height);
        let (_glyphs, width) = render_glyphs(self.font, &self.text, scale, 0.0);

        width
    }

    fn resize(&mut self, rect: Rect<f32>) {
        let mut scale = Scale::uniform(rect.height());
        let max_width = rect.width();

        let (glyphs, width) = render_glyphs(self.font, &self.text, scale, rect.min.x);

        if width <= max_width {
            log::trace!("'{}', resize, using provided scale", self.name);
            self.glyphs = Some(glyphs);
        } else {
            let ratio = max_width / width;
            log::trace!(
                "'{}', resize, area too small scaling to {ratio} times",
                self.name
            );
            debug_assert!(ratio <= 1.0);

            scale = Scale::uniform((rect.height() * ratio).floor());
            log::trace!("'{}' calculated scale: {scale:?}", self.name);
            let (new_glyphs, new_width) = render_glyphs(self.font, &self.text, scale, rect.min.x);

            debug_assert!(new_width <= max_width);

            self.glyphs = Some(new_glyphs);
        }

        self.scale = scale;
        self.text_drawn_last = None;
        self.glyphs_drawn_last = None;
        self.area = rect;
        self.redraw = true;
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        let area: Rect<u32> = self.area.into();
        //log::debug!("'{}', draw, area: {area:?}", self.name);

        if ctx.full_redraw || self.redraw {
            area.draw(self.bg, ctx);
            //area.draw_outline(self.fg, ctx);
        }

        if self.glyphs.is_none() {
            let (glyphs, _width) =
                render_glyphs(self.font, &self.text, self.scale, area.min.x as f32);
            self.glyphs = Some(glyphs);
        }

        let glyphs = self.glyphs.as_ref().unwrap();

        for gly in glyphs {
            if let Some(bb) = gly.pixel_bounding_box() {
                let rect: Rect<u32> = bb.into();

                debug_assert!(self.area.contains_rect(rect.into()));

                ctx.damage.push(rect);
                //rect.draw_outline(self.fg, ctx);
                gly.draw(|x, y, v| {
                    let color = self.bg.blend(self.fg, v);
                    let point = Point::new(rect.min.x + x, rect.min.y + y);

                    debug_assert!(area.contains(point));
                    ctx.put(point, color);
                })
            }
        }

        if ctx.full_redraw || self.redraw {
            self.redraw = false;
            ctx.damage.push(area);
        }

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
        self.text = text;
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
            text_drawn_last: None,
            glyphs: None,
            glyphs_drawn_last: None,
            redraw: true,
        }
    }
}

impl<'glyphs> Default for TextBoxBuilder<'glyphs> {
    fn default() -> Self {
        Self::new()
    }
}
