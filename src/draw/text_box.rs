use crate::draw::*;
use crate::widget::*;

use anyhow::Result;
use rusttype::{Font, PositionedGlyph};

#[derive(Clone)]
pub struct TextBox<'glyphs> {
    font: &'glyphs Font<'glyphs>,

    text: Box<str>,
    text_first_diff: usize,
    name: Box<str>,
    fg: Color,
    bg: Color,

    top_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
    right_margin: u32,
    h_align: Align,
    v_align: Align,

    glyphs_size: Point,
    glyphs: Option<Vec<PositionedGlyph<'glyphs>>>,
    scale: Scale,

    area: Rect,
    desired_text_height: u32,
    desired_width: Option<u32>,

    redraw: bool,
    rerender_text: bool,
}

fn render_glyphs<'a>(
    font: &'a Font<'a>,
    text: &str,
    scale: Scale,
) -> (Vec<PositionedGlyph<'a>>, u32) {
    let v_metrics = font.v_metrics(scale);
    let offset = Point::new(0, v_metrics.ascent.round() as u32);

    let glyphs: Vec<_> = font.layout(text, scale, offset.into()).collect();
    let width = glyphs.last().map_or_else(
        || 0,
        |g| (g.position().x + g.unpositioned().h_metrics().advance_width).ceil() as u32,
    );

    (glyphs, width)
}

impl<'a> TextBox<'a> {
    pub fn set_text(&mut self, new_text: &str) {
        let new_text = new_text.trim();
        if new_text.is_empty() {
            //log::trace!("'{}' set_text :: text set is empty", self.name);
            self.glyphs_size = Point::new(0, 0);
        }
        //log::trace!("'{}' set_text :: text: {new_text}", self.name);

        if let Some((idx, _)) = self
            .text
            .chars()
            .zip(new_text.chars())
            .enumerate()
            .find(|(_idx, (new, old))| new != old)
        {
            self.text_first_diff = idx;
        }

        self.rerender_text |= *self.text != *new_text;
        self.text = new_text.into();
    }

    pub fn set_fg(&mut self, fg: Color) {
        self.redraw = true;
        self.fg = fg;
    }

    pub fn set_bg(&mut self, bg: Color) {
        self.redraw = true;
        self.bg = bg;
    }

    pub fn builder() -> TextBoxBuilder<'a> {
        TextBoxBuilder::new()
    }
}

impl Widget for TextBox<'_> {
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
        self.desired_text_height + self.v_margins()
    }

    fn desired_width(&self, height: u32) -> u32 {
        if let Some(desired_width) = self.desired_width {
            return desired_width;
        }

        if self.text.is_empty() || height == 0 {
            log::debug!("'{}' | desired_width :: nothing to display", self.name);
            return 0;
        }

        let scale =
            Scale::uniform((height - self.v_margins()).min(self.desired_text_height) as f32);
        let (_glyphs, width) = render_glyphs(self.font, &self.text, scale);

        width + self.h_margins()
    }

    fn resize(&mut self, rect: Rect) {
        self.redraw = true;
        if rect == self.area && !self.rerender_text {
            return;
        }
        self.rerender_text = false;

        self.area = rect;
        let width_max =
            (rect.width() - self.h_margins()).clamp(0, self.desired_width.unwrap_or(u32::MAX));
        let height_used = (rect.height() - self.v_margins()).clamp(0, self.desired_text_height);

        if width_max == 0 || height_used == 0 {
            self.glyphs_size = Point::new(0, 0);
            self.glyphs = None;
            return;
        }

        self.scale = Scale::uniform(height_used as f32);

        let (glyphs, width_used) = render_glyphs(self.font, &self.text, self.scale);

        if width_used <= width_max {
            log::debug!("'{}' | resize :: using desired scale", self.name);
            self.glyphs_size = Point::new(width_used, height_used);
            self.glyphs = Some(glyphs);
        } else {
            let ratio = width_max as f32 / width_used as f32;
            debug_assert!((0.0..=1.0).contains(&ratio));

            let height_used_new = (height_used as f32 * ratio).round() as u32;
            let scale_new = Scale::uniform(height_used_new as f32);
            log::debug!(
                "'{}' resize :: scale down by {ratio} from {:?} to {:?}",
                self.name,
                self.scale,
                scale_new
            );
            self.scale = scale_new;
            let (new_glyphs, width_used_new) = render_glyphs(self.font, &self.text, self.scale);

            debug_assert!(width_used_new <= width_max);

            self.glyphs_size = Point::new(width_used_new, height_used_new);
            self.glyphs = Some(new_glyphs);
        };
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if self.glyphs_size.x == 0 || self.glyphs_size.y == 0 {
            return Ok(());
        }

        let redraw_full = ctx.full_redraw || self.redraw;

        if self.text.is_empty() || (!redraw_full && !self.rerender_text) {
            return Ok(());
        }

        if self.rerender_text {
            log::debug!("'{}' | draw :: re-rendering glyphs", self.name);
            let (glyphs, width) = render_glyphs(self.font, &self.text, self.scale);
            if width > self.area.width() {
                self.resize(self.area); // TODO: Make it so we don't re-render twice
            } else {
                self.glyphs = Some(glyphs);
                self.glyphs_size = Point::new(width, self.scale.x as u32);
            }
        }

        assert!(self.glyphs.is_some());
        assert!(self.area.size() >= self.glyphs_size);

        let area_used = self
            .area
            .place_at(self.glyphs_size, self.h_align, self.v_align);
        log::trace!(
            "'{}' | draw :: glyph_size: {}, area_size: {}",
            self.name,
            self.glyphs_size,
            area_used.size()
        );
        assert!(area_used.size() >= self.glyphs_size);

        if redraw_full {
            log::debug!("'{}' | draw :: redrawing fully", self.name);
            self.area.draw(self.bg, ctx);
        }

        let mut bb_last = area_used;
        bb_last.max.x = bb_last.min.x;

        for (idx, gly) in self.glyphs.as_ref().unwrap().iter().enumerate() {
            if let Some(bb_i32) = gly.pixel_bounding_box() {
                let mut bb: Rect = bb_i32.into();
                #[cfg(debug_assertions)]
                {
                    let glyph_width = gly.unpositioned().h_metrics().advance_width.ceil() as u32;
                    //log::trace!(
                    //    "'{}' | draw :: gly: {glyph_width}, bb: {}",
                    //    self.name,
                    //    bb.width()
                    //);
                    assert!(glyph_width >= bb.width());
                }

                if idx == self.text_first_diff && !redraw_full {
                    bb_last.max.x = area_used.max.x;
                    //log::trace!("'{}' | draw :: filling back, {idx}", self.name);
                    bb_last.draw(self.bg, ctx);
                }
                bb.min.y += area_used.min.y;
                bb.max.y += area_used.min.y;
                bb.min.x += area_used.min.x;
                bb.max.x += area_used.min.x;

                //log::trace!("'{}' | draw :: area: {area_used}, bb: {bb}", self.name);
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

        if redraw_full {
            ctx.damage.push(self.area);
        } else if self.rerender_text {
            ctx.damage.push(area_used);
            self.rerender_text = false;
        }

        //self.area.draw_outline(self.fg, ctx);
        //area_used.draw_outline(self.fg, ctx);

        self.text_first_diff = 0;
        self.redraw = false;

        Ok(())
    }
}

impl PositionedWidget for TextBox<'_> {
    fn top_margin(&self) -> u32 {
        self.top_margin
    }
    fn bottom_margin(&self) -> u32 {
        self.bottom_margin
    }
    fn left_margin(&self) -> u32 {
        self.left_margin
    }
    fn right_margin(&self) -> u32 {
        self.right_margin
    }
}

#[derive(Clone)]
pub struct TextBoxBuilder<'glyphs> {
    font: &'glyphs Font<'glyphs>,
    text: Box<str>,
    fg: Color,
    bg: Color,
    desired_text_height: u32,
    desired_width: Option<u32>,

    top_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
    right_margin: u32,
    h_align: Align,
    v_align: Align,
}

impl<'glyphs> TextBoxBuilder<'glyphs> {
    pub fn new() -> TextBoxBuilder<'glyphs> {
        Self {
            font: &FONT,
            fg: color::GOLD,
            bg: color::LOVE,
            desired_text_height: u32::MAX,
            desired_width: None,

            text: Default::default(),
            top_margin: Default::default(),
            bottom_margin: Default::default(),
            left_margin: Default::default(),
            right_margin: Default::default(),
            h_align: Default::default(),
            v_align: Default::default(),
        }
    }

    crate::builder_fields! {
        &'glyphs Font<'glyphs>, font;
        &str, text;
        Color, fg bg;
        u32, desired_text_height desired_width top_margin bottom_margin left_margin right_margin;
        Align, v_align h_align;
    }

    pub fn h_margins(mut self, margin: u32) -> Self {
        self.left_margin = margin / 2;
        self.right_margin = margin / 2;
        self
    }

    pub fn v_margins(mut self, margin: u32) -> Self {
        self.top_margin = margin / 2;
        self.bottom_margin = margin / 2;
        self
    }

    pub fn build(&self, name: &str) -> TextBox<'glyphs> {
        TextBox {
            font: self.font,
            text: self.text.clone(),
            fg: self.fg,
            bg: self.bg,
            desired_text_height: self.desired_text_height,
            desired_width: self.desired_width,
            name: name.into(),

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
            redraw: true,
            rerender_text: true,
            text_first_diff: Default::default(),
        }
    }
}

impl<'glyphs> Default for TextBoxBuilder<'glyphs> {
    fn default() -> Self {
        Self::new()
    }
}
