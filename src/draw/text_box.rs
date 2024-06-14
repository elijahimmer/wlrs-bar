use crate::color::Color;
use crate::draw::*;
use crate::widget::Widget;

use anyhow::Result;
use rusttype::Scale;

#[derive(Clone)]
pub struct TextBox {
    pub scale_provided: Scale,
    pub scale: Scale,
    pub rect: Rect,
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    last_draw_text: String,
    redraw: bool,
}

impl TextBox {
    pub fn new(text: String, scale: Scale, fg: Color, bg: Color) -> Self {
        Self {
            scale_provided: scale,
            scale,
            rect: Rect::default(),
            text,
            fg,
            bg,
            last_draw_text: "".to_string(),
            redraw: false,
        }
    }

    pub fn update_scale(&mut self, rect: Rect) {
        if rect.height() >= self.scale_provided.y as u32
            && rect.width() >= self.scale_provided.x as u32 * self.text.len() as u32 / 2
        {
            log::debug!("text box using provided scale");
            self.scale = self.scale_provided;
        } else {
            log::debug!("text box reducing scale");
            self.scale =
                Scale::uniform((rect.width() / self.text.len() as u32).min(rect.height()) as f32);
        }
    }
}

impl Widget for TextBox {
    fn area(&self) -> Rect {
        self.rect
    }

    fn desired_size(&self) -> Point {
        Point {
            x: self.scale.x as u32 * self.text.len() as u32 / 2, // TODO: why divide by 2?
            y: self.scale.y as u32,
        }
    }

    fn resize(&mut self, rect: Rect) {
        log::debug!("text box resized from {:?} to {:?}", self.rect, rect);
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
            let v_metrics = FONT.v_metrics(self.scale);
            let offset = Point::new(self.rect.min.x, v_metrics.ascent as u32);

            let glyphs: Vec<_> = FONT
                .layout(self.text.as_str(), self.scale, offset.into())
                .collect();

            let glyphs_old: Vec<_> = FONT
                .layout(self.last_draw_text.as_str(), self.scale, offset.into())
                .collect();

            let cur_txt = self.text.as_bytes();
            let old_txt = self.last_draw_text.as_bytes();

            for (idx, gly) in (0..).zip(glyphs.iter()) {
                if cur_txt.get(idx) != old_txt.get(idx) || ctx.full_redraw {
                    if let Some(bb) = gly.pixel_bounding_box() {
                        let rect: Rect = bb.into();
                        if let Some(bb_old) = glyphs_old
                            .get(idx)
                            .and_then(|gly_old| gly_old.pixel_bounding_box())
                        {
                            let bounding = rect.bounding(bb_old.into());
                            debug_assert!(bounding.contains_rect(bb_old.into()));
                            debug_assert!(bounding.contains_rect(rect));

                            ctx.damage.push(bounding);
                            bounding.draw(self.bg, ctx);
                            //self.rect.draw(self.bg, ctx);
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
