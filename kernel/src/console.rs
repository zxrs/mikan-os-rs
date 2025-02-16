use crate::{
    Result, TextWriter,
    fonts::write_ascii,
    frame_buffer::{PixelWriter, Rgb},
};

const ROWS: usize = 25;
const COLS: usize = 80;

pub struct Console<W: PixelWriter> {
    writer: W,
    fg_color: Rgb,
    bg_color: Rgb,
    buffer: [[char; COLS + 1]; ROWS],
    cursor_row: usize,
    cursor_col: usize,
}

/// TOOD!
/// Need to rewrite!
/// Console struct should be implemented for core::fmt::Write...
impl<W: PixelWriter> Console<W> {
    pub fn new(writer: W, fg_color: Rgb, bg_color: Rgb) -> Self {
        Self {
            writer,
            fg_color,
            bg_color,
            buffer: [['\0'; COLS + 1]; ROWS],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn put_string(&mut self, s: &str) -> Result<()> {
        s.chars().try_for_each(|c| {
            if c.eq(&'\n') {
                self.new_line()?;
            } else if self.cursor_col < COLS - 1 {
                write_ascii(
                    &mut self.writer,
                    8 * self.cursor_col as u32,
                    16 * self.cursor_row as u32,
                    c,
                    self.fg_color,
                )?;
                self.cursor_col += 1;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn new_line(&self) -> Result<()> {
        todo!()
    }
}
