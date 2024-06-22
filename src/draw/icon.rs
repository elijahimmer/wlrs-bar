use super::prelude::*;
use crate::widget::{ClickType, PositionedWidget, Widget};
use anyhow::Result;

use rusttype::{Font, PositionedGlyph, Scale};

/// A single character displayed as large as possible
pub struct Icon<'glyph> {
    font: &'glyph Font<'glyph>,

    icon: char,
    name: Box<str>,

    fg_drawn: Color,
    bg_drawn: Color,

    fg: Color,
    bg: Color,

    /// ratio of height to top_margin
    top_margin: f32,
    /// ratio of height to bottom_margin
    bottom_margin: f32,
    /// ratio of height to left_margin
    left_margin: f32,
    /// ratio of height to right_margin
    right_margin: f32,

    h_align: Align,
    v_align: Align,

    glyph: Option<(PositionedGlyph<'glyph>, Point)>,

    area: Rect,
    desired_height: Option<u32>,
    desired_width: Option<u32>,
}

fn render_icon<'glyph, 'name>(
    name: &'name str,
    font: &'glyph Font<'glyph>,
    icon: char,
    max_size: Point,
) -> (PositionedGlyph<'glyph>, Point) {
    let Point {
        x: max_width,
        y: max_height,
    } = max_size;

    let scale = Scale::uniform(max_height as f32);

    let offset = rusttype::point(0.0, 0.0);

    let glyph = font.glyph(icon);
    let positioned_glyph = glyph.clone().scaled(scale).positioned(offset);
    let bb: Rect = {
        let mut bb = positioned_glyph
            .pixel_bounding_box()
            .expect("Glyph should have a bounding box");

        bb.max.y -= bb.min.y;
        bb.min.y = 0;

        bb.max.x -= bb.min.x;
        bb.min.x = 0;

        bb.into()
    };

    let Point {
        x: bb_width,
        y: bb_height,
    } = bb.size();

    // the scale to reach the max width/height
    let max_width_scale = ((max_width as f32).powi(2) / bb_width as f32).round() - 1.0;
    let max_height_scale = ((max_height as f32).powi(2) / (bb_height + 1) as f32).round() - 1.0;

    let new_scale = Scale::uniform(max_width_scale.min(max_height_scale));
    #[cfg(feature = "icon-logs")]
    log::trace!(
        "'{name}' | render_glyph :: width scale: {max_width_scale}, height scale: {max_height_scale}, min_scale: {}",
        new_scale.x
    );

    let new_glyph = glyph.scaled(new_scale).positioned(offset);
    let new_bb: Rect = {
        let mut new = new_glyph.clone().pixel_bounding_box().unwrap();

        new.max.y -= new.min.y;
        new.min.y = 0;

        new.max.x -= new.min.x;
        new.min.x = 0;

        new.into()
    };

    #[cfg(feature = "icon-logs")]
    log::trace!(
        "'{name}' | render_glyph :: max width: {max_width}, glyph width: {}",
        bb.max.x,
    );
    #[cfg(feature = "icon-logs")]
    log::trace!(
        "'{name}' | render_glyph :: max height: {max_height}, glyph height: {}",
        bb.max.y,
    );
    assert!(
        bb.size() <= max_size,
        "'{name}' | render_glyph :: bb: {}, max_size: {}",
        bb.size(),
        max_size
    );

    (new_glyph, new_bb.max)
}

impl Icon<'_> {
    pub fn builder<'a>() -> IconBuilder<'a> {
        IconBuilder::new()
    }
}

impl Widget for Icon<'_> {
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
        self.desired_height
            .unwrap_or(u32::MAX)
            .saturating_add(self.v_margins())
    }

    fn desired_width(&self, height: u32) -> u32 {
        if let Some(desired_width) = self.desired_width {
            return desired_width;
        }

        let size_used = Point {
            x: u32::MAX,
            y: height.min(self.desired_height.unwrap_or(u32::MAX)),
        };
        let (
            _glyphs,
            Point {
                x: glyph_width,
                y: glyph_height,
            },
        ) = render_icon(&self.name, self.font, self.icon, size_used);
        assert!(glyph_height <= height);

        glyph_width + self.h_margins()
    }

    fn resize(&mut self, new_area: Rect) {
        self.area = new_area;
        let Point {
            x: width,
            y: height,
        } = new_area.size();
        let used_size = Point {
            x: width.saturating_sub(self.h_margins()),
            y: height
                .min(self.desired_height.unwrap_or(u32::MAX))
                .saturating_sub(self.v_margins()),
        };
        let glyph = render_icon(&self.name, self.font, self.icon, used_size);
        assert!(
            glyph.1 <= used_size,
            "'{}' :: glyph size: {}, max size: {}, area: {}",
            self.name,
            glyph.1,
            used_size,
            new_area.size(),
        );

        self.glyph = Some(glyph);
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if !ctx.full_redraw {
            return Ok(());
        }
        let (gly, size) = self.glyph.as_ref().unwrap();

        #[cfg(feature = "icon-logs")]
        log::trace!(
            "'{}' | draw :: area: {}, size: {}",
            self.name,
            self.area.size(),
            *size
        );

        let Point {
            x: max_width,
            y: max_height,
        } = self.area.size();

        let area_used = Rect::new(
            self.area.min + Point::new(self.left_margin(), self.top_margin()),
            self.area.max - Point::new(self.right_margin(), self.bottom_margin()),
        );
        assert!(
            self.area.contains_rect(area_used),
            "area doesn't contain area used. area: {}, area_used: {}",
            self.area,
            area_used
        );

        let bb = area_used.place_at(*size, self.h_align, self.v_align);

        #[cfg(feature = "icon-logs")]
        log::trace!("'{}' | draw :: bb: {bb}, area: {}", self.name, self.area);

        gly.draw(|x, y, v| {
            let point @ Point { x, y } = bb.min + Point::new(x, y);

            let idx = 4 * (x + y * ctx.rect.width()) as usize;

            let screen_bytes: &mut [u8; 4] = (&mut ctx.canvas[idx..idx + 4]).try_into().unwrap();

            let existing_color = Color::from_argb8888(screen_bytes);
            let color = self
                .bg_drawn
                .composite(existing_color)
                .blend(self.fg_drawn, v);

            *screen_bytes = color.argb8888();

            assert!(
                self.area.contains(point),
                "glyph not contained in area: {}, point: {point}",
                self.area
            );
        });

        #[cfg(feature = "icon-outlines")]
        self.area.draw_outline(super::color::PINE, ctx);
        #[cfg(feature = "icon-outlines")]
        bb.draw_outline(super::color::IRIS, ctx);

        Ok(())
    }

    fn click(&mut self, _button: ClickType, _point: Point) -> Result<()> {
        todo!()
    }

    fn motion(&mut self, _point: Point) -> Result<()> {
        todo!()
    }

    fn motion_leave(&mut self, _point: Point) -> Result<()> {
        todo!()
    }
}

impl PositionedWidget for Icon<'_> {
    fn top_margin(&self) -> u32 {
        (self.area().height() as f32 * self.top_margin) as u32
    }
    fn bottom_margin(&self) -> u32 {
        (self.area().height() as f32 * self.bottom_margin) as u32
    }
    fn left_margin(&self) -> u32 {
        (self.area().height() as f32 * self.left_margin) as u32
    }
    fn right_margin(&self) -> u32 {
        (self.area().height() as f32 * self.right_margin) as u32
    }
}

#[derive(Clone)]
pub struct IconBuilder<'glyph> {
    font: &'glyph Font<'glyph>,
    icon: char,
    fg: Color,
    bg: Color,
    desired_height: Option<u32>,
    desired_width: Option<u32>,

    /// ratio of height to top_margin
    top_margin: f32,
    /// ratio of height to bottom_margin
    bottom_margin: f32,
    /// ratio of height to left_margin
    left_margin: f32,
    /// ratio of height to right_margin
    right_margin: f32,

    h_align: Align,
    v_align: Align,
}

impl<'glyph> IconBuilder<'glyph> {
    pub fn new() -> IconBuilder<'glyph> {
        Self {
            font: &FONT,

            top_margin: 0.0,
            bottom_margin: 0.0,
            left_margin: 0.0,
            right_margin: 0.0,

            desired_height: Default::default(),
            desired_width: Default::default(),
            fg: Default::default(),
            bg: Default::default(),
            icon: Default::default(),
            h_align: Default::default(),
            v_align: Default::default(),
        }
    }

    crate::builder_fields! {
        &'glyph Font<'glyph>, font;
        u32, desired_height desired_width;
        f32, top_margin bottom_margin left_margin right_margin;
        Color, fg bg;
        Align, v_align h_align;
        char, icon;
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

    pub fn build(&self, name: &str) -> Icon<'glyph> {
        Icon {
            font: self.font,
            icon: self.icon,
            fg_drawn: self.fg,
            bg_drawn: self.bg,
            fg: self.fg,
            bg: self.bg,
            desired_height: self.desired_height,
            desired_width: self.desired_width,
            name: name.into(),

            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            h_align: self.h_align,
            v_align: self.v_align,

            area: Default::default(),
            glyph: Default::default(),
        }
    }
}

impl<'glyph> Default for IconBuilder<'glyph> {
    fn default() -> Self {
        Self::new()
    }
}
