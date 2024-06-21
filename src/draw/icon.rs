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

    top_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
    right_margin: u32,
    h_align: Align,
    v_align: Align,

    glyph: Option<(PositionedGlyph<'glyph>, Point)>,

    area: Rect,
    desired_text_height: Option<u32>,
    desired_width: Option<u32>,
}

fn render_icon<'glyph>(
    font: &'glyph Font<'glyph>,
    icon: char,
    max_size: Point,
) -> (PositionedGlyph, Point) {
    let Point { y: max_height, .. } = max_size;

    let scale = Scale::uniform(max_height as f32);

    let v_metrics = font.v_metrics(scale);
    let offset = Point::new(0, v_metrics.ascent.round() as u32);

    let glyph = font.glyph(icon);
    let positioned_glyph = glyph.clone().scaled(scale).positioned(offset.into());
    let bb: Rect = positioned_glyph
        .pixel_bounding_box()
        .expect("Glyph should have a bounding box")
        .into();

    let Point {
        x: _bb_width,
        y: bb_height,
    } = bb.size();

    //let width_ratio = bb_width as f32 / max_width as f32;
    //let height_ratio = bb_height as f32 / max_height as f32;
    //assert!((0.0..=1.0).contains(&width_ratio));
    //assert!((0.0..=1.0).contains(&height_ratio));

    //let (min_ratio, max_ratio) = crate::utils::cmp(width_ratio, height_ratio);

    let new_scale = Scale::uniform(((max_height * max_height) / (1 + bb_height)) as f32 + 1.0);

    let new_glyph = glyph.scaled(new_scale).positioned(offset.into());
    let mut new_bb = new_glyph.clone().pixel_bounding_box().unwrap();

    new_bb.max.y -= new_bb.min.y;
    new_bb.min.y = 0;

    new_bb.max.x -= new_bb.min.x;
    new_bb.min.x = 0;

    let new_bb: Rect = new_bb.into();
    assert!(new_bb.min == Point {x: 0, y: 0}, "new_bb: {}", new_bb);

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
        return u32::MAX;
        self.desired_text_height
            .unwrap_or(u32::MAX)
            .saturating_add(self.v_margins())
    }

    fn desired_width(&self, height: u32) -> u32 {
        if let Some(desired_width) = self.desired_width {
            return desired_width;
        }

        todo!()
    }

    fn resize(&mut self, new_area: Rect) {
        self.area = new_area;
        let Point {x, y} = new_area.size();
        let used_size = Point {x: x.min(self.desired_text_height.unwrap_or(u32::MAX)), y };
        self.glyph = Some(render_icon(self.font, self.icon, used_size));
    }

    fn draw(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        if !ctx.full_redraw {
            return Ok(());
        }
        let (gly, size) = self.glyph.as_ref().unwrap();

        let center_shift = (self.area.size() - *size) / 2;

        let bb = Rect::new(self.area.min, self.area.min + Point {x: size.x, y: size.y }).x_shift(center_shift.x as i32).y_shift(center_shift.y as i32);

        log::trace!("bb: {bb}, area: {}", self.area);

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
        bb.draw_outline(super::color::IRIS, ctx);

        Ok(())
    }

    fn click(&mut self, _button: ClickType, _point: Point) -> Result<()> {
        todo!()
    }

    fn motion(&mut self, point: Point) -> Result<()> {
        todo!()
    }

    fn motion_leave(&mut self, point: Point) -> Result<()> {
        todo!()
    }
}

impl PositionedWidget for Icon<'_> {
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
pub struct IconBuilder<'glyph> {
    font: &'glyph Font<'glyph>,
    icon: char,
    fg: Color,
    bg: Color,
    desired_text_height: Option<u32>,
    desired_width: Option<u32>,

    top_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
    right_margin: u32,
    h_align: Align,
    v_align: Align,
}

impl<'glyph> IconBuilder<'glyph> {
    pub fn new() -> IconBuilder<'glyph> {
        Self {
            font: &FONT,

            desired_text_height: Default::default(),
            desired_width: Default::default(),
            fg: Default::default(),
            bg: Default::default(),
            icon: Default::default(),
            top_margin: Default::default(),
            bottom_margin: Default::default(),
            left_margin: Default::default(),
            right_margin: Default::default(),
            h_align: Default::default(),
            v_align: Default::default(),
        }
    }

    crate::builder_fields! {
        &'glyph Font<'glyph>, font;
        u32, desired_text_height desired_width top_margin bottom_margin left_margin right_margin;
        Color, fg bg;
        Align, v_align h_align;
        char, icon;
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

    pub fn build(&self, name: &str) -> Icon<'glyph> {
        Icon {
            font: self.font,
            icon: self.icon,
            fg_drawn: self.fg,
            bg_drawn: self.bg,
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

//fn render_glyphs<'a>(font: &'a Font<'a>, text: &str, scale: Scale) {
//    let v_metrics = font.v_metrics(scale);
//    let offset = Point::new(0, v_metrics.ascent.round() as u32);
//
//    let glyphs = font
//        .layout(text, scale, offset.into())
//        .filter_map(|gly| gly.pixel_bounding_box().map(|bb| (gly, Rect::from(bb))))
//        .collect::<Vec<_>>();
//
//    let width = glyphs.last().map_or_else(
//        || 0,
//        |(g, _bb)| (g.position().x + g.unpositioned().h_metrics().advance_width).ceil() as u32,
//    );
//    let height: u32 = glyphs
//        .iter()
//        .map(|(_g, bb)| (bb.max.y - bb.min.y))
//        .max()
//        .unwrap_or(0);
//
//    (glyphs, Point::new(width, height))
//}
//
//fn render_glyphs_maximize<'a>(
//    font: &'a Font<'a>,
//    text: &str,
//    height: u32,
//    maximize_space: bool,
//) -> (Vec<Glyph<'a>>, Point, u32, Scale) {
//    let scale = Scale::uniform(height as f32);
//
//    let (
//        glyphs,
//        size @ Point {
//            y: height_used,
//            ..
//        },
//    ) = render_glyphs(font, text, scale);
//    assert!(height_used <= height, "{}:{} :: {height_used} > {height}", file!(), line!());
//
//    if !maximize_space {
//        #[cfg(feature = "textbox-logs")]
//        log::debug!("render_glyphs_maximize :: scale determined: {scale:?}");
//        (glyphs, size, 0, scale)
//    } else {
//        #[cfg(feature = "textbox-logs")]
//        log::debug!("render_glyphs_maximize :: height: {height} height_used: {height_used}");
//        let scale_height_new = ((height as f32).powf(2.0) / (height_used + 1) as f32).round();
//        let scale_new = Scale::uniform(scale_height_new);
//        #[cfg(feature = "textbox-logs")]
//        log::debug!("render_glyphs_maximize :: rescaling {scale:?} to {scale_new:?}");
//
//        let (
//            glyphs_new,
//            size_new @ Point {
//                y: height_new,
//                ..
//            },
//        ) = render_glyphs(font, text, scale_new);
//
//        assert!(height_new <= height);
//        let height_offset = (scale_height_new.floor() as u32 - height_new) / 2;
//
//        (glyphs_new, size_new, height_offset, scale)
//    }
//}
