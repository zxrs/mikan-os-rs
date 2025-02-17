use crate::{
    Result,
    fonts::write_ascii,
    frame_buffer::{PixelWriter, Rgb},
};
use core::{fmt, str};

const ROWS: usize = 25;
const COLS: usize = 80;

pub struct Console<W: PixelWriter> {
    writer: W,
    fg_color: Rgb,
    bg_color: Rgb,
    buffer: [[char; COLS]; ROWS],
    cursor_row: usize,
    cursor_col: usize,
}

impl<W: PixelWriter> fmt::Write for Console<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().try_for_each(|c| {
            if c.eq(&'\n') {
                self.new_line().map_err(|_| fmt::Error)?;
            } else if self.cursor_col < COLS {
                write_ascii(
                    &mut self.writer,
                    8 * self.cursor_col as u32,
                    16 * self.cursor_row as u32,
                    c,
                    self.fg_color,
                )
                .map_err(|_| fmt::Error)?;
                self.buffer[self.cursor_row][self.cursor_col] = c;
                self.cursor_col += 1;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl<W: PixelWriter> Console<W> {
    pub fn new(writer: W, fg_color: Rgb, bg_color: Rgb) -> Self {
        Self {
            writer,
            fg_color,
            bg_color,
            buffer: [['\0'; COLS]; ROWS],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    fn new_line(&mut self) -> Result<()> {
        self.cursor_col = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            (0..ROWS * 16).try_for_each(|y| {
                (0..COLS * 8).try_for_each(|x| {
                    self.writer.write(x as u32, y as u32, self.bg_color)?;
                    Ok(())
                })?;
                Ok(())
            })?;
            self.buffer.rotate_left(1);
            if let Some(last_row) = self.buffer.last_mut() {
                last_row.fill('\0');
            };

            self.buffer.iter().enumerate().try_for_each(|(j, row)| {
                row.iter().enumerate().try_for_each(|(i, c)| {
                    write_ascii(
                        &mut self.writer,
                        i as u32 * 8,
                        j as u32 * 16,
                        *c,
                        self.fg_color,
                    )?;
                    Ok(())
                })?;
                Ok(())
            })?;
        }
        Ok(())
    }
}
