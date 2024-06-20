
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
) -> (Vec<Glyph<'a>>, Point, u32, Scale) {
    let scale = Scale::uniform(height as f32);

    let (
        glyphs,
        size @ Point {
            y: height_used,
            ..
        },
    ) = render_glyphs(font, text, scale);
    assert!(height_used <= height, "{}:{} :: {height_used} > {height}", file!(), line!());

    if !maximize_space {
        #[cfg(feature = "textbox-logs")]
        log::debug!("render_glyphs_maximize :: scale determined: {scale:?}");
        (glyphs, size, 0, scale)
    } else {
        #[cfg(feature = "textbox-logs")]
        log::debug!("render_glyphs_maximize :: height: {height} height_used: {height_used}");
        let scale_height_new = ((height as f32).powf(2.0) / (height_used + 1) as f32).round();
        let scale_new = Scale::uniform(scale_height_new);
        #[cfg(feature = "textbox-logs")]
        log::debug!("render_glyphs_maximize :: rescaling {scale:?} to {scale_new:?}");

        let (
            glyphs_new,
            size_new @ Point {
                y: height_new,
                ..
            },
        ) = render_glyphs(font, text, scale_new);

        assert!(height_new <= height);
        let height_offset = (scale_height_new.floor() as u32 - height_new) / 2;

        (glyphs_new, size_new, height_offset, scale)
    }
}
