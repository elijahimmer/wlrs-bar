use crate::draw::*;
use crate::log::*;
use crate::widget::*;

use anyhow::Result;
use rusttype::{Font, PositionedGlyph, Scale};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum RedrawState {
    #[default]
    None,
    Full,
    Partial(NonZeroUsize),
}

#[derive(Clone)]
pub struct TextBox {
    font: Font<'static>,

    text: Box<str>,
    lc: LC,

    fg_drawn: Color,
    bg_drawn: Color,

    fg: Color,
    bg: Color,
    hover_fg: Option<Color>,
    hover_bg: Option<Color>,

    top_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
    right_margin: u32,
    h_align: Align,
    v_align: Align,

    glyphs_size: Option<Point>,
    glyphs: Option<Vec<(PositionedGlyph<'static>, Rect)>>,

    area: Rect,
    desired_text_height: u32,
    desired_width: Option<u32>,

    redraw: RedrawState,
}

impl TextBox {
    fn render_glyphs(&self, height: u32) -> (Vec<(PositionedGlyph<'static>, Rect)>, Point) {
        let scale = Scale::uniform(height as f32);

        let v_metrics = self.font.v_metrics(scale);
        let offset = Point {
            x: 0,
            y: v_metrics.ascent.round() as u32,
        };

        let glyphs = self
            .font
            .layout(&self.text, scale, offset.into())
            .filter_map(|gly| gly.pixel_bounding_box().map(|bb| (gly, Rect::from(bb))))
            .collect::<Vec<_>>();

        let width = glyphs.last().map_or_else(
            || 0,
            |(g, _bb)| (g.position().x + g.unpositioned().h_metrics().advance_width).ceil() as u32,
        );
        let height: u32 = glyphs
            .iter()
            .map(|(_g, bb)| (bb.max.y - bb.min.y))
            .max()
            .unwrap_or(0);

        (
            glyphs,
            Point {
                x: width,
                y: height,
            },
        )
    }
    pub fn set_text(&mut self, new_text: &str) {
        let new_text = new_text.trim();
        if new_text.is_empty() {
            if self.lc.should_log {
                debug!("{} set_text :: text set is empty", self.lc);
            }
            self.glyphs_size = None;
            self.glyphs = None;
            return;
        }

        match self
            .text
            .chars()
            .zip(new_text.chars())
            .position(|(new, old)| new != old)
        {
            Some(idx) => {
                self.redraw = match (NonZeroUsize::new(idx), self.redraw) {
                    (None, _) => RedrawState::Full,
                    (Some(_), RedrawState::Full) => RedrawState::Full,
                    (Some(idx), RedrawState::Partial(jdx)) => RedrawState::Partial(idx.min(jdx)),
                    (Some(idx), RedrawState::None) => RedrawState::Partial(idx),
                }
            }

            None => return,
        }
        self.text = new_text.into();
        if self.lc.should_log {
            trace!("{} | set_text :: new_text: '{new_text}'", self.lc);
        }

        let area_height = self.area.height().min(self.desired_text_height);

        if self.lc.should_log {
            debug!("{} | set_text :: re-rendering glyphs", self.lc);
        }
        let (glyphs, glyphs_size @ Point { x: width, .. }) = self.render_glyphs(area_height);
        if width > self.area.width() {
            if self.lc.should_log {
                info!("{} set_text :: resorting to resize before write", self.lc);
            }
            self.resize(self.area); // TODO: Make it so we don't re-render like 4 times
        } else {
            self.glyphs = Some(glyphs);
            self.glyphs_size = Some(Point {
                x: glyphs_size.x,
                y: area_height,
            });
        }
    }

    pub fn set_fg(&mut self, fg: Color) {
        if fg != self.fg {
            self.redraw = RedrawState::Full;
            if self.fg_drawn == self.fg {
                self.fg_drawn = fg;
            }
            self.fg = fg;
        }
    }

    pub fn set_bg(&mut self, bg: Color) {
        if bg != self.bg {
            self.redraw = RedrawState::Full;
            if self.bg_drawn == self.bg {
                self.bg_drawn = bg;
            }
            self.bg = bg;
        }
    }

    pub fn builder() -> TextBoxBuilder<NeedsFont> {
        TextBoxBuilder::<NeedsFont>::new()
    }
}

impl Widget for TextBox {
    fn lc(&self) -> &LC {
        &self.lc
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
            if self.lc.should_log {
                debug!("{} | desired_width :: nothing to display", self.lc);
            }
            return 0;
        }

        let (_glyphs, Point { x: width, .. }, ..) =
            self.render_glyphs(height.min(self.desired_text_height));

        width + self.h_margins()
    }

    fn resize(&mut self, new_area: Rect) {
        if new_area == self.area {
            if self.lc.should_log {
                debug!("{} | resize :: area didn't change", self.lc);
            }
            return;
        }

        self.redraw = RedrawState::Full;
        if self.lc.should_log {
            trace!("{} | resize :: new_area: {new_area}", self.lc);
        }
        let old_area = self.area;
        self.area = new_area;

        if new_area.size() == old_area.size() {
            if self.lc.should_log {
                trace!(
                    "{} | resize :: box was moved, not resized, not re-rendering text",
                    self.lc
                );
            }
            return;
        }
        if self.lc.should_log {
            trace!("{} | resize :: re-rendering text", self.lc);
        }

        // the maximum area the text can be (while following margins)
        let area_max = self
            .area
            .shrink_top(self.top_margin())
            .shrink_bottom(self.bottom_margin())
            .shrink_left(self.left_margin())
            .shrink_right(self.right_margin());

        let area_max_size @ Point {
            x: width_max,
            y: area_max_height,
        } = area_max.size();

        let height_max = area_max_height.min(self.desired_text_height);

        let (glyphs, glyphs_size @ Point { x: width_used, .. }) = self.render_glyphs(height_max);

        if width_used <= width_max {
            if self.lc.should_log {
                debug!("{} | resize :: using desired height: {height_max}", self.lc);
            }

            assert!(
                glyphs_size <= area_max_size,
                "text rendered was too tall. max: {area_max_size}, rendered: {glyphs_size}"
            );
            self.glyphs_size = Some(Point {
                x: glyphs_size.x,
                y: height_max,
            });
            // uses height max as the glyphs rely on that for placement
            self.glyphs = Some(glyphs);
        } else {
            // it was too big
            let ratio = width_max as f32 / width_used as f32;
            assert!(
                (0.0..=1.0).contains(&ratio),
                "ratio of {width_max}/{width_used} = {ratio} wasn't between 0 and 1."
            );

            let height_new = (height_max as f32 * ratio).round() as u32;

            if self.lc.should_log {
                debug!(
                    "{} resize :: scale down by {ratio}, {height_max:?} -> {height_new:?}",
                    self.lc,
                );
            }

            let (glyphs_new, glyphs_size_new) = self.render_glyphs(height_new);
            assert!(glyphs_size_new <= area_max_size, "the text scaled down was still too large. max: {area_max_size}, rendered: {glyphs_size_new}");

            self.glyphs_size = Some(Point {
                x: glyphs_size_new.x,
                y: height_max,
            });
            self.glyphs = Some(glyphs_new);
        }
    }

    fn should_redraw(&mut self) -> bool {
        self.glyphs_size.is_some() && self.redraw != RedrawState::None
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if self.lc.should_log {
            trace!(
                "{} | draw :: redraw: {:?}, full redraw: {}",
                self.lc,
                self.redraw,
                ctx.full_redraw
            );
        }

        let area = self.area;

        let area_used = area.place_at(self.glyphs_size.unwrap(), self.h_align, self.v_align);
        let area_used_size = area_used.size();
        if self.lc.should_log {
            trace!(
                "{} | draw :: area_used: {area_used} size: {area_used_size}",
                self.lc
            );
        }
        let glyphs_size = self.glyphs_size.unwrap();

        assert!(area_used_size >= glyphs_size);

        let glyphs = self.glyphs.as_ref().unwrap();

        let glyph_skip_count = match self.redraw {
            RedrawState::Full | RedrawState::None => {
                if self.lc.should_log {
                    debug!("{} | draw :: redrawing fully, at {}", self.lc, self.area);
                }
                self.area.draw_composite(self.bg_drawn, ctx);
                ctx.damage.push(area);
                0
            }
            RedrawState::Partial(idx) => {
                if self.lc.should_log {
                    debug!(
                        "{} | draw :: Partial Redraw from idx: {}",
                        self.lc,
                        usize::from(idx),
                    );
                }
                let mut area_to_fill = area_used;
                area_to_fill.min.x += glyphs[usize::from(idx) - 1]
                    .0
                    .unpositioned()
                    .h_metrics()
                    .advance_width
                    .ceil() as u32;

                area_to_fill.draw_composite(self.bg_drawn, ctx);
                ctx.damage.push(area_to_fill);
                idx.into()
            }
        };

        glyphs
            .iter()
            .skip(glyph_skip_count)
            .for_each(|(gly, bb_unshifted)| {
                if self.lc.should_log {
                    trace!("{} | draw :: bb-unshifted: {bb_unshifted}", self.lc);
                }
                let bb_x_shifted = bb_unshifted.x_shift(area_used.min.x as i32);
                let bb = bb_x_shifted.y_shift(area_used.min.y as i32);
                if self.lc.should_log {
                    trace!("{} | draw :: bb: {bb}", self.lc);
                }
                assert!(
                    bb.size() <= glyphs_size,
                    "bb is too big: bb: {bb}, maximum glyph size: {glyphs_size}"
                );
                assert!(
                    area_used.contains_rect(bb),
                    "bb not in area: {area_used}, bb: {bb}"
                );
                gly.draw(|x, y, v| {
                    let point @ Point { x, y } = bb.min + Point { x, y };

                    let idx = 4 * (x + y * ctx.rect.width()) as usize;

                    let screen_bytes: &mut [u8; 4] =
                        (&mut ctx.canvas[idx..idx + 4]).try_into().unwrap();

                    let existing_color = Color::from_argb8888(screen_bytes);
                    let color = self
                        .bg_drawn
                        .composite(existing_color)
                        .blend(self.fg_drawn, v);

                    *screen_bytes = color.argb8888();

                    assert!(
                        area_used.contains(point),
                        "glyph not contained in area: {area_used}, point: {point}"
                    );
                });

                #[cfg(feature = "textbox-outlines-bounding")]
                bb.draw_outline(color::IRIS, ctx);
            });

        #[cfg(feature = "textbox-outlines-area")]
        self.area.draw_outline(color::PINE, ctx);
        #[cfg(feature = "textbox-outlines-area")]
        ctx.damage.push(self.area);

        #[cfg(feature = "textbox-outlines-used")]
        area_used.draw_outline(color::GOLD, ctx);
        #[cfg(feature = "textbox-outlines-used")]
        ctx.damage.push(area_used);

        //#[cfg(feature = "textbox-outlines-text")]
        //text_area.draw_outline(color::LOVE, ctx);
        //#[cfg(feature = "textbox-outlines-text")]
        //ctx.damage.push(text_area);

        self.redraw = RedrawState::None;

        Ok(())
    }

    fn click(&mut self, _button: ClickType, _point: Point) -> Result<()> {
        Ok(())
    }

    fn motion(&mut self, point: Point) -> Result<()> {
        if self.lc.should_log {
            debug!("{} | motion :: Point: {point}", self.lc);
        }
        assert!(self.area.contains(point));

        if let Some(c) = self.hover_fg.filter(|&c| c != self.fg_drawn) {
            self.redraw = RedrawState::Full;
            self.fg_drawn = c;
        }

        if let Some(c) = self.hover_bg.filter(|&c| c != self.bg_drawn) {
            self.redraw = RedrawState::Full;
            self.bg_drawn = c;
        }

        Ok(())
    }

    fn motion_leave(&mut self, _point: Point) -> Result<()> {
        if self.lc.should_log {
            debug!("{} | motion_leave :: Point: {_point}", self.lc);
        }

        if self.fg != self.fg_drawn {
            self.redraw = RedrawState::Full;
            self.fg_drawn = self.fg;
        }

        if self.bg != self.bg_drawn {
            self.redraw = RedrawState::Full;
            self.bg_drawn = self.bg;
        }

        Ok(())
    }
}

impl PositionedWidget for TextBox {
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

#[derive(Clone, Default)]
pub struct TextBoxBuilder<T> {
    font: Option<Font<'static>>,

    text: Box<str>,
    fg: Color,
    bg: Color,
    hover_fg: Option<Color>,
    hover_bg: Option<Color>,
    desired_text_height: Option<u32>,
    desired_width: Option<u32>,

    top_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
    right_margin: u32,
    h_align: Align,
    v_align: Align,

    _state: PhantomData<T>,
}

#[derive(Clone, Debug, Default)]
pub struct NeedsFont {}
#[derive(Clone, Debug, Default)]
pub struct HasFont {}

impl<T> TextBoxBuilder<T> {
    pub fn new() -> TextBoxBuilder<NeedsFont> {
        Default::default()
    }
    pub fn font(self, font: Font<'static>) -> TextBoxBuilder<HasFont> {
        TextBoxBuilder {
            font: Some(font),
            _state: PhantomData::<HasFont> {},

            text: self.text,
            fg: self.fg,
            bg: self.bg,
            hover_fg: self.hover_fg,
            hover_bg: self.hover_bg,
            desired_text_height: self.desired_text_height,
            desired_width: self.desired_width,

            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            h_align: self.h_align,
            v_align: self.v_align,
        }
    }
    crate::builder_fields! {
        u32, desired_text_height desired_width top_margin bottom_margin left_margin right_margin;
        Color, fg bg hover_fg hover_bg;
        Align, v_align h_align;
        &str, text;
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
}

impl TextBoxBuilder<HasFont> {
    pub fn build(&self, lc: LC) -> TextBox {
        TextBox {
            font: self.font.to_owned().expect("should be impossible"),
            text: self.text.clone(),
            fg_drawn: self.fg,
            bg_drawn: self.bg,
            fg: self.fg,
            bg: self.bg,
            hover_fg: self.hover_fg,
            hover_bg: self.hover_bg,
            desired_text_height: self.desired_text_height.unwrap_or(u32::MAX),
            desired_width: self.desired_width,
            lc,

            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            h_align: self.h_align,
            v_align: self.v_align,

            area: Default::default(),
            glyphs: Default::default(),
            glyphs_size: Default::default(),
            redraw: Default::default(),
        }
    }
}
