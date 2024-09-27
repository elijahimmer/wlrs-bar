pub mod color;
pub mod icon;
pub mod point;
pub mod prelude;
pub mod progress;
pub mod rect;
pub mod text_box;

use prelude::*;

pub const DEFAULT_FONT_DATA: &[u8] = include_bytes!("../../fonts/FiraCodeNerdFontMono-Regular.ttf");
pub const DEFAULT_FONT_INDEX: u32 = 0;

// which edge to align to
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub enum Align {
    Start,
    End,
    #[default]
    Center,
    CenterAt(f32),
}

use smithay_client_toolkit::shm::slot::Buffer;
pub struct DrawCtx<'ctx> {
    pub damage: &'ctx mut Vec<Rect>,
    pub buffer: &'ctx Buffer,
    pub canvas: &'ctx mut [u8],
    pub rect: Rect,
    pub full_redraw: bool,
}

impl DrawCtx<'_> {
    pub fn put(&mut self, pnt: Point, color: Color) {
        assert!(self.rect.contains(pnt));

        let idx: usize = 4 * (pnt.x + pnt.y * self.rect.width()) as usize;

        let array: &mut [u8; 4] = (&mut self.canvas[idx..idx + 4]).try_into().unwrap();
        *array = color.argb8888();
    }

    pub fn put_composite(&mut self, pnt: Point, color: Color) {
        assert!(self.rect.contains(pnt));

        let idx: usize = 4 * (pnt.x + pnt.y * self.rect.width()) as usize;

        let array: &mut [u8; 4] = (&mut self.canvas[idx..idx + 4]).try_into().unwrap();
        let existing_color = Color::from_argb8888(array);

        let composite = color.composite(existing_color);
        *array = composite.argb8888();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash, Default)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}
