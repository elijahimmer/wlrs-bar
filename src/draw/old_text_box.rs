use crate::color::Color;
use crate::draw::*;
use crate::widget::Widget;

use anyhow::Result;
use rusttype::{PositionedGlyph, Scale};

#[derive(Clone)]
pub struct TextBox<'a> {
    scale_provided: Scale,
    scale: Scale,
    pub rect: Rect,
    pub fg: Color,
    pub bg: Color,

    glyphs: Vec<PositionedGlyph<'a>>,
    pub text: String,
    glyphs_old: Option<Vec<PositionedGlyph<'a>>>,
    last_draw_text: String,

    name: &'static str,
    redraw: bool,
}

impl<'a> TextBox<'a> {
    pub fn new(
        name: &'static str,
        text: String,
        pixel_height: f32,
        fg: Color,
        bg: Color,
    ) -> TextBox<'a> {
        let scale = Scale::uniform(FONT.scale_for_pixel_height(pixel_height));
        let mut me = Self {
            scale_provided: scale,
            scale,
            rect: Rect::default(),
            fg,
            bg,
            name,
            text,
            glyphs: vec![],
            last_draw_text: "".to_string(),
            glyphs_old: None,
            redraw: false,
        };
        me.update_glyphs();
        me
    }

    pub fn update_scale(&mut self, rect: Rect) {
        if rect.height() >= self.scale_provided.y as u32
            && rect.width() >= self.scale_provided.x as u32 * self.text.len() as u32 / 2
        {
            log::debug!("text box '{}' using provided scale", self.name);
            self.scale = self.scale_provided;
        } else {
            log::debug!("text box '{}' reducing scale", self.name);
            self.scale =
                Scale::uniform((rect.width() / self.text.len() as u32).min(rect.height()) as f32);
        }
    }

    pub fn update_glyphs(&mut self) {
        let v_metrics = FONT.v_metrics(self.scale);
        let offset = Point::new(self.rect.min.x, v_metrics.ascent as u32);

        let glyphs: Vec<_> = FONT
            .layout(self.text.as_str(), self.scale, offset.into())
            .collect();

        self.glyphs = glyphs;
    }
}

impl<'a> Widget for TextBox<'a> {
    fn area(&self) -> Rect {
        self.rect
    }

    fn desired_size(&self) -> Point {
        //self.update_glyphs();
        let v_metrics = FONT.v_metrics(self.scale);

        Point {
            x: self
                .glyphs
                .last()
                .unwrap()
                .pixel_bounding_box()
                .unwrap()
                .max
                .x as u32,
            y: v_metrics.ascent as u32,
        }
    }

    fn resize(&mut self, rect: Rect) {
        log::debug!(
            "text box '{}' resized from {:?} to {:?}",
            self.name,
            self.rect,
            rect
        );
        self.update_scale(rect);
        self.rect = rect;
        self.redraw = true;
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if ctx.full_redraw || self.redraw {
            self.redraw = false;
            self.last_draw_text = "".to_string();
        } else if self.last_draw_text == self.text {
            return Ok(());
        }

        {
            self.update_glyphs();
            let glyphs = &self.glyphs;

            let cur_txt = self.text.as_bytes();
            let old_txt = self.last_draw_text.as_bytes();

            for (idx, gly) in (0..).zip(glyphs.iter()) {
                if cur_txt.get(idx) != old_txt.get(idx) || ctx.full_redraw {
                    if let Some(bb) = gly.pixel_bounding_box() {
                        let rect: Rect = bb.into();
                        if let Some(bb_old) = self
                            .glyphs_old
                            .as_ref()
                            .and_then(|glys| glys.get(idx))
                            .and_then(|gly_old| gly_old.pixel_bounding_box())
                        {
                            let bounding = rect.largest(bb_old.into());
                            debug_assert!(bounding.contains_rect(bb_old.into()));
                            debug_assert!(bounding.contains_rect(rect));

                            ctx.damage.push(bounding);
                            bounding.draw(self.bg, ctx);
                        } else {
                            ctx.damage.push(rect);
                        }

                        gly.draw(|x, y, v| {
                            ctx.put(Point { x, y } + rect.min, self.bg.blend(self.fg, v));
                        });
                    } else {
                        log::warn!(
                            "glyph has no bounding box? char: '{}'",
                            *cur_txt.get(idx).unwrap() as char
                        );
                    }
                }
            }

            self.last_draw_text = self.text.clone();
        }

        Ok(())
    }
}
