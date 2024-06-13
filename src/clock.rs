use anyhow::Result;
use chrono::Timelike;
use rusttype::Scale;

use crate::color;
use crate::draw::FONT;
use crate::widget::{Point, Rect};

//pub struct TwoDigits {
//    pub dims: Rect,
//    pub value: u8,
//}

#[derive(Clone, Copy)]
pub struct Clock {
    pub scale: Scale,
    pub dims: Rect,
    //pub hours: TwoDigits,
    //pub minutes: TwoDigits,
    //pub seconds: TwoDigits,
}

pub const NUM_CHARS: u32 = 8;

impl crate::widget::Widget for Clock {
    fn new(dims: Rect) -> Self {
        log::info!("Initalizing Clock at dims: {dims:?}");
        let scale = Scale::uniform(32.0); //(dims.width / NUM_CHARS).min(dims.height) as f32);

        let time = chrono::Local::now();

        Self {
            scale,
            dims,
            //hours: time.hour(),
            //minutes: time.minute(),
            //seconds: time.second(),
        }
    }
    fn recommended_size(container: Rect) -> Point {
        let scale_factor = (container.width / NUM_CHARS).min(container.height);
        let size: Point = (
            (scale_factor * NUM_CHARS).min(container.width) / 2, // todo: Find out why I have to divide
            // by 2 here...
            scale_factor,
        )
            .into(); // 8 for the number of characters

        debug_assert!(container.width >= size.x);
        debug_assert!(container.height >= size.y);

        size
    }

    fn desired_size(&self, container: Rect) -> Point {
        Self::recommended_size(container)
    }
    fn draw(&mut self, canvas: &mut [u8], canvas_size: Point) -> Result<()> {
        let (width, height) = (canvas_size.x, canvas_size.y);
        let v_metrics = FONT.v_metrics(self.scale);
        let offset = Point::new(self.dims.x, v_metrics.ascent as u32);

        let time = chrono::Local::now();
        let time_str = format!(
            "{:02}{:02}{:02}",
            time.hour(),
            time.minute(),
            time.second()
        );

        let glyphs: Vec<_> = FONT.layout(&time_str, self.scale, offset.into()).collect();

        for gly in glyphs {
            if let Some(bb) = gly.pixel_bounding_box() {
                let rect: Rect = bb.into();
                rect.fill(*color::LOVE, canvas, canvas_size);
                gly.draw(|x, y, v| {
                    let start_x = x as i32 + bb.min.x;
                    let start_y = y as i32 + bb.min.y;

                    // bounds check the indexes
                    if start_x >= 0
                        && start_x < width as i32
                        && start_y >= 0
                        && start_y < height as i32
                    {
                        let start_idx: usize =
                            4 * (start_x as usize + start_y as usize * width as usize);

                        let argb = color::BASE.blend(*color::LOVE, v);

                        let array: &mut [u8; 4] =
                            (&mut canvas[start_idx..start_idx + 4]).try_into().unwrap();
                        *array = argb.argb8888();
                    }
                });
            } else {
                log::warn!("glyph has no bounding box?")
            }
        }

        Ok(())
    }

    fn update_dimensions(&mut self, dimensions: Rect) {
        log::info!("Resizing clock from: {:?}, to: {:?}", self.dims, dimensions);
        self.dims = dimensions;
        self.scale = Scale::uniform(32.0); //(self.dims.width / NUM_CHARS).min(self.dims.height) as f32);
    }
}
