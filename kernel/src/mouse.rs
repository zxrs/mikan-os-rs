use crate::{
    Result, Rgb, Vector2D,
    frame_buffer::{BGRPixelWriter, PixelWriter},
};

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
#[rustfmt::skip]
const MOUSE_CURSOR_SHAPE: [&str; MOUSE_CURSOR_HEIGHT] = [
   //0123456789abcde
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];

pub fn draw_mouse_cursor<W: PixelWriter>(
    pixel_writer: &mut W,
    position: Vector2D<i32>,
) -> Result<()> {
    (0..MOUSE_CURSOR_HEIGHT).try_for_each(|dy| {
        (0..MOUSE_CURSOR_WIDTH).try_for_each(|dx| {
            if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).eq(&Some('@')) {
                pixel_writer.write(
                    (position.x + dx as i32) as u32,
                    (position.y + dy as i32) as u32,
                    Rgb::black(),
                )
            } else if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).eq(&Some('.')) {
                pixel_writer.write(
                    (position.x + dx as i32) as u32,
                    (position.y + dy as i32) as u32,
                    Rgb::white(),
                )
            } else {
                Ok(())
            }
        })
    })?;
    Ok(())
}

pub fn erace_mouse_cursor<W: PixelWriter>(
    pixel_writer: &mut W,
    position: Vector2D<i32>,
    erasr_color: Rgb,
) -> Result<()> {
    (0..MOUSE_CURSOR_HEIGHT).try_for_each(|dy| {
        (0..MOUSE_CURSOR_WIDTH).try_for_each(|dx| {
            if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).ne(&Some(' ')) {
                pixel_writer.write(
                    (position.x + dx as i32) as u32,
                    (position.y + dy as i32) as u32,
                    erasr_color,
                )
            } else {
                Ok(())
            }
        })
    })?;
    Ok(())
}

pub struct MouseCursor {
    pixel_writer: &'static mut BGRPixelWriter,
    erase_color: Rgb,
    position: Vector2D<i32>,
}

impl MouseCursor {
    pub fn new(
        pixel_writer: &'static mut BGRPixelWriter,
        erase_color: Rgb,
        initial_position: Vector2D<i32>,
    ) -> Result<Self> {
        draw_mouse_cursor(pixel_writer, Vector2D::new(200, 100))?;
        Ok(Self {
            pixel_writer,
            erase_color,
            position: initial_position,
        })
    }

    pub fn move_relative(&mut self, displacement: Vector2D<i32>) -> Result<()> {
        erace_mouse_cursor(self.pixel_writer, self.position, self.erase_color)?;
        self.position += displacement;
        draw_mouse_cursor(self.pixel_writer, self.position)?;
        Ok(())
    }
}
