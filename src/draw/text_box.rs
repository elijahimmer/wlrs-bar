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

    maximize_space: bool,
    height_offset: u32,

    area: Rect,
    desired_text_height: u32,
    desired_width: Option<u32>,

    redraw: bool,
    rerender_text: bool,
}

fn render_glyphs<'a>(font: &'a Font<'a>, text: &str, scale: Scale) -> (Vec<Glyph<'a>>, Point) {
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
    let height: u32 = glyphs
        .iter()
        .map(|(_g, bb)| (bb.max.y - bb.min.y))
        .max()
        .unwrap_or(0);

    (glyphs, Point::new(width, height))
}

fn render_glyphs_maximize<'a>(
    font: &'a Font<'a>,
    text: &str,
    height: u32,
    maximize_space: bool,
) -> (Vec<Glyph<'a>>, u32, u32, Scale) {
    let scale = Scale::uniform(height as f32);

    let (
        glyphs,
        Point {
            x: width_used,
            y: height_used,
        },
    ) = render_glyphs(font, text, scale);
    assert!(height_used <= height, "{height_used} > {height}");

    if !maximize_space || (height * 8 / 10) <= height_used {
        #[cfg(feature = "textbox-logs")]
        log::debug!("render_glyphs_maximize :: scale determined: {scale:?}");
        (glyphs, width_used, 0, scale)
    } else {
        #[cfg(feature = "textbox-logs")]
        log::debug!("render_glyphs_maximize :: height: {height} height_used: {height_used}");
        let scale_height_new = ((height as f32).powf(2.0) / (height_used + 1) as f32).floor();
        let scale_new = Scale::uniform(scale_height_new);
        #[cfg(feature = "textbox-logs")]
        log::debug!("render_glyphs_maximize :: rescaling {scale:?} to {scale_new:?}");

        let (
            glyphs_new,
            Point {
                x: width_new,
                y: height_new,
            },
        ) = render_glyphs(font, text, scale_new);

        assert!(height_new <= height, "{height_new} > {height}");
        let height_offset = (scale_height_new.floor() as u32 - height_new) / 2;

        (glyphs_new, width_new, height_offset, scale)
    }
}

impl<'a> TextBox<'a> {
    pub fn set_text(&mut self, new_text: &str) {
        let new_text = new_text.trim();
        if new_text.is_empty() {
            #[cfg(feature = "textbox-logs")]
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
            #[cfg(feature = "textbox-logs")]
            log::debug!("'{}' | desired_width :: nothing to display", self.name);
            return 0;
        }

        let (_glyphs, width, _height_offset, _scale) =
            render_glyphs_maximize(self.font, &self.text, height, self.maximize_space);

        width + self.h_margins()
    }

    fn resize(&mut self, rect: Rect) {
        if rect == self.area && !self.rerender_text {
            #[cfg(feature = "textbox-logs")]
            log::warn!("'{}' | resize :: resized for no reason", self.name);
            return;
        }
            #[cfg(feature = "textbox-logs")]
        log::trace!("'{}' | resize :: rect: {rect}", self.name);
        self.redraw = true;
        self.rerender_text = false;
        self.area = rect;

        let width_max =
            (rect.width() - self.h_margins()).min(self.desired_width.unwrap_or(u32::MAX));
        let height_max = (rect.height() - self.v_margins()).min(self.desired_text_height);

        if width_max == 0 || height_max == 0 {
            self.glyphs_size = None;
            self.glyphs = None;
            return;
        }

        let (glyphs, width_used, height_offset, scale) =
            render_glyphs_maximize(self.font, &self.text, height_max, self.maximize_space);
        self.height_offset = height_offset;

        if width_used <= width_max {
            #[cfg(feature = "textbox-logs")]
            log::debug!(
                "'{}' | resize :: using desired height: {}",
                self.name,
                height_max
            );
            self.glyphs_size = Some(Point::new(width_used, height_max)); // use height max as BBs
                                                                         // rely on that
            self.glyphs = Some(glyphs);
        } else {
            // it was too big
            let ratio = width_max as f32 / width_used as f32;
            debug_assert!((0.0..=1.0).contains(&ratio));

            let scale_new = Scale::uniform((scale.x * ratio).round());

            #[cfg(feature = "textbox-logs")]
            log::debug!(
                "'{}' resize :: scale down by {ratio}, {scale:?} -> {scale_new:?}",
                self.name,
            );

            let (
                new_glyphs,
                Point {
                    x: width_used_new,
                    y: height_used_new,
                },
            ) = render_glyphs(self.font, &self.text, scale_new);
            debug_assert!(height_used_new <= height_max);
            debug_assert!(width_used_new <= width_max);

            self.glyphs_size = Some(Point::new(width_used_new, height_max));
            self.glyphs = Some(new_glyphs);
        }
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        let redraw_full = ctx.full_redraw || self.redraw;

        if self.glyphs_size.is_none() || !(redraw_full || self.rerender_text) {
            return Ok(());
        }

        if self.rerender_text {
            // TODO: Optimize so you only re-render what has changed, if applicable
            #[cfg(feature = "textbox-logs")]
            log::debug!("'{}' | draw :: re-rendering glyphs", self.name);
            let (glyphs, width, height_offset, _scale) = render_glyphs_maximize(
                self.font,
                &self.text,
                self.area.height(),
                self.maximize_space,
            );
            self.height_offset = height_offset;
            if width > self.area.width() {
                log::info!("'{}' draw :: resorting to resize before write", self.name);
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
        let glyphs = self.glyphs.as_ref().unwrap();

        if redraw_full {
            #[cfg(feature = "textbox-logs")]
            log::debug!(
                "'{}' | draw :: redrawing fully, at {}",
                self.name,
                self.area
            );
            self.area.draw_composite(self.bg_drawn, ctx);
        } else if self.text_first_diff == 0 {
            self.area.draw_composite(self.bg_drawn, ctx);
            ctx.damage.push(self.area);
        } else {
            let mut area_to_fill = self.area;
            area_to_fill.min.x += glyphs[self.text_first_diff - 1].0.unpositioned().h_metrics().advance_width.ceil() as u32;

            area_to_fill.draw_composite(self.bg_drawn, ctx);
            ctx.damage.push(area_to_fill);
        }

        glyphs
            .iter()
            .skip(self.text_first_diff)
            .for_each(|(gly, mut bb)| {
                bb.min += area_used.min;
                bb.max += area_used.min;
            #[cfg(feature = "textbox-logs")]
                log::trace!(
                    "'{}' | draw :: bb-pre: {bb}, height_offset: {}",
                    self.name,
                    self.height_offset
                );
                bb = bb.y_shift(-(self.height_offset as i32));
            #[cfg(feature = "textbox-logs")]
                log::trace!("'{}' | draw :: bb: {bb}, area_used: {area_used}", self.name);
                gly.draw(|x, y, v| {
                    let point = Point::new(bb.min.x + x, bb.min.y + y);
                    let idx: usize = 4 * (point.x + point.y * ctx.rect.width()) as usize;
                    let screen_bytes: &mut [u8; 4] = (&mut ctx.canvas[idx..idx + 4]).try_into().unwrap();

                    let existing_color = Color::from_argb8888(screen_bytes);
                    //assert!(existing_color == color::SURFACE, "{existing_color} != {}", color::SURFACE);
                    let color = self.bg.composite(existing_color).blend(self.fg_drawn, v);

                    *screen_bytes = color.argb8888();

                    debug_assert!(
                        area_used.contains(point),
                        "glyph not contained in area: {area_used}, point: {point}"
                    );
                });

            #[cfg(feature = "textbox-outlines")]
                bb.draw_outline(color::IRIS, ctx);
            });

        if cfg!(feature = "textbox-outlines") {
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

    maximize_space: bool,
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

            maximize_space: false,
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
        u32, desired_text_height desired_width top_margin bottom_margin left_margin right_margin;
        Color, fg bg hover_fg hover_bg;
        Align, v_align h_align;
        bool, maximize_space;
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
            maximize_space: self.maximize_space,

            redraw: true,
            rerender_text: true,

            height_offset: Default::default(),
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
