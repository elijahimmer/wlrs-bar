use crate::color::{self, Color};
use crate::draw::*;
use crate::widget::*;

use anyhow::Result;
use rusttype::{Font, PositionedGlyph};

#[derive(Clone)]
pub struct TextBox<'glyphs> {
    font: &'glyphs Font<'glyphs>,

    text: String,
    text_first_diff: usize,
    name: Box<str>,
    fg: Color,
    bg: Color,

    top_margin: f32,
    bottom_margin: f32,
    left_margin: f32,
    right_margin: f32,
    h_align: Align,
    v_align: Align,

    glyphs_size: Point<f32>,
    glyphs: Option<Vec<PositionedGlyph<'glyphs>>>,
    scale: Scale,

    area: Rect<f32>,
    desired_text_height: f32,
    desired_width: Option<f32>,

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

    (glyphs, width.round())
}

impl<'a> TextBox<'a> {
    pub fn set_text(&mut self, new_text: &str) {
        for (idx, (new, old)) in self.text.chars().zip(new_text.chars()).enumerate() {
            if new != old {
                self.text_first_diff = idx;
                break;
            }
        }
        let new_text = new_text.trim();
        if new_text.is_empty() {
            self.glyphs_size = Point::new(0.0, 0.0);
        }
        self.rerender_text = self.text != new_text;
        self.text = new_text.to_string();
    }

    pub fn set_fg(&mut self, fg: Color) {
        self.redraw = true;
        self.fg = fg;
    }

    pub fn set_bg(&mut self, bg: Color) {
        self.redraw = true;
        self.fg = bg;
    }

    pub fn builder() -> TextBoxBuilder<'a> {
        TextBoxBuilder::new()
    }
}

impl Widget for TextBox<'_> {
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
        self.desired_text_height + self.v_margins()
    }

    fn desired_width(&self, height: f32) -> f32 {
        if let Some(desired_width) = self.desired_width {
            return desired_width;
        }

        if self.text.is_empty() || height <= 0.0 {
            log::debug!("'{}', desired_width, nothing to display", self.name);
            return 0.0;
        }

        let scale =
            Scale::uniform((height - self.v_margins()).clamp(0.0, self.desired_text_height));
        let (_glyphs, width) = render_glyphs(self.font, &self.text, scale);

        width + self.h_margins()
    }

    fn resize(&mut self, rect: Rect<f32>) {
        self.area = rect;
        self.redraw = true;
        self.rerender_text = false;

        let width_max = (rect.width() - self.h_margins())
            .clamp(0.0, self.desired_width.unwrap_or(f32::INFINITY));
        let height_used = (rect.height() - self.v_margins()).clamp(0.0, self.desired_text_height);

        if width_max <= 0.0 || height_used <= 0.0 {
            self.glyphs_size = Point::new(0.0, 0.0);
            self.glyphs = None;
            return;
        }

        self.scale = Scale::uniform(height_used);

        let (glyphs, width_used) = render_glyphs(self.font, &self.text, self.scale);

        if width_used.round() <= width_max.round() {
            log::debug!("'{}', resize, using desired scale", self.name);
            self.glyphs_size = Point::new(width_used, height_used);
            self.glyphs = Some(glyphs);
        } else {
            let ratio = width_max / width_used;
            debug_assert!((0.0..=1.0).contains(&ratio));

            let height_used_new = (height_used * ratio).floor();
            let scale_new = Scale::uniform(height_used_new);
            log::debug!(
                "'{}', scale down by {ratio} from {:?} to {:?}",
                self.name,
                self.scale,
                scale_new
            );
            self.scale = scale_new;
            let (new_glyphs, width_used_new) = render_glyphs(self.font, &self.text, self.scale);

            debug_assert!(width_used_new.round() <= width_max.round());

            self.glyphs_size = Point::new(width_used_new, height_used_new);
            self.glyphs = Some(new_glyphs);
        };
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        let area: Rect<u32> = self.area.into();
        if self.glyphs_size.x <= 0.0 || self.glyphs_size.y <= 0.0 {
            // glyphs are 0 width
            return Ok(());
        }

        let redraw_full = ctx.full_redraw || self.redraw;

        if !redraw_full && !self.rerender_text || self.text.is_empty() {
            return Ok(());
        }

        assert!(!self.text.is_empty());
        assert!(self.glyphs.is_some());
        assert!(self.area.size() >= self.glyphs_size);

        let glyphs_size: Point<u32> = self.glyphs_size.into();
        let area_used = area.place_at(glyphs_size, self.h_align, self.v_align);
        //log::trace!(
        //    "'{}', glyph_size: {:?}, area_size: {:?}",
        //    self.name,
        //    glyphs_size,
        //    area_used.size()
        //);
        assert!(area_used.size() >= glyphs_size);

        if self.rerender_text {
            //log::trace!("'{}', draw, re-rendering glyphs", self.name);
            let (glyphs, width) = render_glyphs(self.font, &self.text, self.scale);
            self.glyphs = Some(glyphs);
            self.glyphs_size = Point::new(width, area_used.height() as f32);
        }
        if redraw_full {
            log::trace!("'{}', draw, redrawing fully", self.name);
            area.draw(self.bg, ctx);
        }

        let mut bb_last = area_used;
        bb_last.max.x = bb_last.min.x;

        for (idx, gly) in self.glyphs.as_ref().unwrap().iter().enumerate() {
            if let Some(bb_i32) = gly.pixel_bounding_box() {
                let mut bb: Rect<u32> = bb_i32.into();
                #[cfg(debug_assertions)]
                {
                    let glyph_width = gly.unpositioned().h_metrics().advance_width.round();
                    //log::trace!("'{}', gly: {glyph_width}, bb: {}", self.name, bb.width());
                    assert!(glyph_width as u32 >= bb.width());
                }

                if idx == self.text_first_diff && !redraw_full {
                    bb_last.max.x = area_used.max.x;
                    //log::trace!("'{}', draw, filling back, {idx}", self.name);
                    bb_last.draw(self.bg, ctx);
                }
                bb.min.y += area_used.min.y;
                bb.max.y += area_used.min.y;
                bb.min.x += area_used.min.x;
                bb.max.x += area_used.min.x;

                //log::trace!("'{}', area: {area_used:?}, bb: {bb:?}", self.name);
                debug_assert!(area_used.contains_rect(bb));

                ctx.damage.push(bb);
                gly.draw(|x, y, v| {
                    let color = self.bg.blend(self.fg, v);
                    let point = Point::new(bb.min.x + x, bb.min.y + y);

                    debug_assert!(area_used.contains(point));
                    ctx.put(point, color);
                });
                bb_last.min.x = bb.max.x;
                //bb.draw_outline(self.fg, ctx);
            }
        }

        if self.rerender_text {
            ctx.damage.push(area_used);
        }

        //area.draw_outline(self.fg, ctx);
        //area_used.draw_outline(self.fg, ctx);

        self.text_first_diff = 0;
        self.redraw = false;

        Ok(())
    }
}

impl PositionedWidget for TextBox<'_> {
    fn top_margin(&self) -> f32 {
        self.top_margin
    }
    fn bottom_margin(&self) -> f32 {
        self.bottom_margin
    }
    fn left_margin(&self) -> f32 {
        self.left_margin
    }
    fn right_margin(&self) -> f32 {
        self.right_margin
    }
}

#[derive(Clone)]
pub struct TextBoxBuilder<'glyphs> {
    font: &'glyphs Font<'glyphs>,
    text: String,
    fg: Color,
    bg: Color,
    desired_text_height: f32,
    desired_width: Option<f32>,

    top_margin: f32,
    bottom_margin: f32,
    left_margin: f32,
    right_margin: f32,
    h_align: Align,
    v_align: Align,
}

impl<'glyphs> TextBoxBuilder<'glyphs> {
    pub fn new() -> TextBoxBuilder<'glyphs> {
        Self {
            font: &FONT,
            text: String::new(),
            fg: *color::LOVE,
            bg: *color::SURFACE,
            desired_text_height: f32::INFINITY,
            desired_width: None,

            top_margin: Default::default(),
            bottom_margin: Default::default(),
            left_margin: Default::default(),
            right_margin: Default::default(),
            h_align: Default::default(),
            v_align: Default::default(),
        }
    }

    crate::builder_fields! {
        &'glyphs Font<'glyphs>, font,
        String, text,
        Color, fg bg,
        f32, desired_text_height top_margin bottom_margin left_margin right_margin,
        Align, v_align h_align
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn h_margins(mut self, margin: f32) -> Self {
        self.left_margin = margin / 2.0;
        self.right_margin = margin / 2.0;
        self
    }

    pub fn v_margins(mut self, margin: f32) -> Self {
        self.top_margin = margin / 2.0;
        self.bottom_margin = margin / 2.0;
        self
    }

    pub fn build(&self, name: Box<str>) -> TextBox<'glyphs> {
        TextBox {
            font: self.font,
            text: self.text.to_owned(),
            fg: self.fg,
            bg: self.bg,
            desired_text_height: self.desired_text_height,
            desired_width: self.desired_width,
            name,

            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            h_align: self.h_align,
            v_align: self.v_align,

            scale: Scale::uniform(0.0),
            area: Default::default(),
            glyphs: None,
            glyphs_size: Default::default(),
            redraw: false,
            rerender_text: false,
            text_first_diff: Default::default(),
        }
    }
}

impl<'glyphs> Default for TextBoxBuilder<'glyphs> {
    fn default() -> Self {
        Self::new()
    }
}