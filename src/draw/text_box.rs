use crate::draw::*;
use crate::widget::*;

use anyhow::Result;
use rusttype::{Font, PositionedGlyph};

type Glyph<'glyphs> = (PositionedGlyph<'glyphs>, Rect);

#[derive(Clone)]
pub struct TextBox<'glyphs> {
    font: &'glyphs Font<'glyphs>,

    text: Box<str>,
    text_first_diff: usize, // TODO: Change to a full range and if aligned to the right render
    // from the right back. That'll be fun :)
    name: Box<str>,

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
    glyphs: Option<Vec<Glyph<'glyphs>>>,
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
) -> (Vec<(PositionedGlyph<'a>, Rect)>, u32) {
    let v_metrics = font.v_metrics(scale);
    let offset = Point::new(0, v_metrics.ascent.round() as u32);

    let glyphs = font
        .layout(text, scale, offset.into())
        .filter_map(|gly| gly.pixel_bounding_box().map(|bb| (gly, Rect::from(bb))))
        .collect::<Vec<_>>();
    let width = glyphs.last().map_or_else(
        || 0,
        |(g, _bb)| (g.position().x + g.unpositioned().h_metrics().advance_width).ceil() as u32,
    );

    (glyphs, width)
}

impl<'a> TextBox<'a> {
    pub fn set_text(&mut self, new_text: &str) {
        let new_text = new_text.trim();
        if new_text.is_empty() {
            //#[cfg(feature = "debug-textbox")]
            log::debug!("'{}' set_text :: text set is empty", self.name);
            self.glyphs_size = None;
            self.glyphs = None;
            return;
        }

        if !(self.redraw || self.rerender_text) {
            if let Some((idx, _)) = self
                .text
                .chars()
                .zip(new_text.chars())
                .enumerate()
                .find(|(_idx, (new, old))| new != old)
            {
                self.text_first_diff = idx;
            }
        }

        self.rerender_text |= *self.text != *new_text;
        self.text = new_text.into();
    }

    pub fn set_fg(&mut self, fg: Color) {
        if fg != self.fg {
            self.redraw = true;
            if self.fg_drawn == self.fg {
                self.fg_drawn = fg;
            }
            self.fg = fg;
        }
    }

    pub fn set_bg(&mut self, bg: Color) {
        if bg != self.bg {
            self.redraw = true;
            if self.bg_drawn == self.bg {
                self.bg_drawn = bg;
            }
            self.bg = bg;
        }
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
            #[cfg(feature = "debug-textbox")]
            log::debug!("'{}' | desired_width :: nothing to display", self.name);
            return 0;
        }

        let scale =
            Scale::uniform((height - self.v_margins()).min(self.desired_text_height) as f32);
        let (_glyphs, width) = render_glyphs(self.font, &self.text, scale);

        width + self.h_margins()
    }

    fn resize(&mut self, rect: Rect) {
        if rect == self.area && !self.rerender_text {
            #[cfg(feature = "debug-textbox-resize")]
            log::warn!("'{}' | resize :: resized for no reason", self.name);
            return;
        }
        #[cfg(feature = "debug-textbox-resize")]
        log::trace!("'{}' | resize :: rect: {rect}", self.name);
        self.redraw = true;
        self.rerender_text = false;
        self.area = rect;

        let width_max =
            (rect.width() - self.h_margins()).min(self.desired_width.unwrap_or(u32::MAX));
        let height_used = (rect.height() - self.v_margins()).min(self.desired_text_height);

        if width_max == 0 || height_used == 0 {
            self.glyphs_size = None;
            self.glyphs = None;
            return;
        }

        self.scale = Scale::uniform(height_used as f32);

        let (glyphs, width_used) = render_glyphs(self.font, &self.text, self.scale);

        if width_used <= width_max {
            #[cfg(feature = "debug-textbox-resize")]
            log::debug!("'{}' | resize :: using desired scale", self.name);
            self.glyphs_size = Some(Point::new(width_used, height_used));
            self.glyphs = Some(glyphs);
        } else {
            let ratio = width_max as f32 / width_used as f32;
            debug_assert!((0.0..=1.0).contains(&ratio));

            let height_used_new = (height_used as f32 * ratio).round() as u32;
            let scale_new = Scale::uniform(height_used_new as f32);
            #[cfg(feature = "debug-textbox-resize")]
            log::debug!(
                "'{}' resize :: scale down by {ratio} from {:?} to {:?}",
                self.name,
                self.scale,
                scale_new
            );
            self.scale = scale_new;
            let (new_glyphs, width_used_new) = render_glyphs(self.font, &self.text, self.scale);

            debug_assert!(width_used_new <= width_max);

            self.glyphs_size = Some(Point::new(width_used_new, height_used_new));
            self.glyphs = Some(new_glyphs);
        };
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        let redraw_full = ctx.full_redraw || self.redraw;

        if self.glyphs_size.is_none() || !(redraw_full || self.rerender_text) {
            return Ok(());
        }

        if self.rerender_text {
            // TODO: Optimize so you only re-render what has changed, if applicable
            #[cfg(feature = "debug-textbox-draw")]
            log::debug!("'{}' | draw :: re-rendering glyphs", self.name);
            let (glyphs, width) = render_glyphs(self.font, &self.text, self.scale);
            if width > self.area.width() {
                self.resize(self.area); // TODO: Make it so we don't re-render twice
            } else {
                self.glyphs = Some(glyphs);
                self.glyphs_size = Some(Point::new(width, self.glyphs_size.unwrap().y));
            }
        }

        let glyphs_size = self.glyphs_size.unwrap();

        assert!(self.area.size() >= glyphs_size);
        assert!(self.area.size().x >= glyphs_size.x + self.h_margins());

        let text_area = Rect::new(
            self.area.min + Point::new(self.left_margin, self.top_margin),
            self.area.max - Point::new(self.right_margin, self.bottom_margin),
        );

        let area_used = text_area.place_at(glyphs_size, self.h_align, self.v_align);

        if redraw_full {
            //#[cfg(feature = "debug-textbox-draw")]
            log::debug!(
                "'{}' | draw :: redrawing fully, at {}",
                self.name,
                self.area
            );
            self.area.draw(self.bg_drawn, ctx);
        }

        let glyphs = self.glyphs.as_ref().unwrap();

        if self.text_first_diff == 0 {
            self.area.draw(self.bg_drawn, ctx);
            ctx.damage.push(self.area);
        } else {
            let mut area_to_fill = self.area;
            area_to_fill.min.x += glyphs[self.text_first_diff - 1].1.max.x;
            area_to_fill.draw(self.bg_drawn, ctx);
            ctx.damage.push(area_to_fill);
        }

        glyphs
            .iter()
            .skip(self.text_first_diff)
            .for_each(|(gly, mut bb)| {
                bb.min += area_used.min;
                gly.draw(|x, y, v| {
                    let color = self.bg_drawn.blend(self.fg_drawn, v);
                    let point = Point::new(bb.min.x + x, bb.min.y + y);

                    debug_assert!(area_used.contains(point));
                    ctx.put(point, color);
                });
            });

        if cfg!(feature = "debug-textbox-draw") {
            self.area.draw_outline(color::PINE, ctx);
            area_used.draw_outline(color::GOLD, ctx);
            text_area.draw_outline(color::LOVE, ctx);
            ctx.damage.push(self.area);
            ctx.damage.push(area_used);
            ctx.damage.push(text_area);
        }

        self.text_first_diff = 0;
        self.redraw = false;
        self.rerender_text = false;

        Ok(())
    }

    fn click(&mut self, _button: ClickType, _point: Point) -> Result<()> {
        Ok(())
    }

    fn motion(&mut self, point: Point) -> Result<()> {
        //log::debug!("'{}' | motion :: Point: {point}", self.name);
        debug_assert!(self.area.contains(point));

        if let Some(c) = self.hover_fg.filter(|&c| c != self.fg_drawn) {
            self.redraw = true;
            self.fg_drawn = c;
        }

        if let Some(c) = self.hover_bg.filter(|&c| c != self.bg_drawn) {
            self.redraw = true;
            self.bg_drawn = c;
        }

        Ok(())
    }

    fn motion_leave(&mut self, point: Point) -> Result<()> {
        log::debug!("'{}' | motion_leave :: Point: {point}", self.name);

        if self.fg != self.fg_drawn {
            self.redraw = true;
            self.fg_drawn = self.fg;
        }

        if self.bg != self.bg_drawn {
            self.redraw = true;
            self.bg_drawn = self.bg;
        }

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
    hover_fg: Option<Color>,
    hover_bg: Option<Color>,
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
            desired_text_height: u32::MAX,
            desired_width: None,

            fg: Default::default(),
            bg: Default::default(),
            hover_fg: Default::default(),
            hover_bg: Default::default(),
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
        Color, fg bg hover_fg hover_bg;
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
            fg_drawn: self.fg,
            bg_drawn: self.bg,
            fg: self.fg,
            bg: self.bg,
            hover_fg: self.hover_fg,
            hover_bg: self.hover_bg,
            desired_text_height: self.desired_text_height,
            desired_width: self.desired_width,
            name: name.into(),

            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            h_align: self.h_align,
            v_align: self.v_align,

            redraw: true,
            rerender_text: true,
            scale: Scale::uniform(0.0),

            area: Default::default(),
            glyphs: Default::default(),
            glyphs_size: Default::default(),
            text_first_diff: Default::default(),
        }
    }
}

impl<'glyphs> Default for TextBoxBuilder<'glyphs> {
    fn default() -> Self {
        Self::new()
    }
}
