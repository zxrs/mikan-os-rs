use crate::{PixelWriter, Result, Rgb};

pub fn write_ascii<W: PixelWriter>(
    writer: &mut W,
    x: u32,
    y: u32,
    c: char,
    rgb: Rgb,
) -> Result<()> {
    if c.ne(&'A') {
        unimplemented!();
    }
    (0..16).try_for_each(|dy| -> Result<()> {
        (0..8).try_for_each(|dx| -> Result<()> {
            if ((FONT_A[dy] << dx) & 0x80) > 0 {
                writer.write(x + dx, y + dy as u32, rgb)?;
            }
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(())
}

#[rustfmt::skip]
const FONT_A: &[u8] = &[
    0b00000000, //
    0b00011000, //    **
    0b00011000, //    **
    0b00011000, //    **
    0b00011000, //    **
    0b00100100, //   *  *
    0b00100100, //   *  *
    0b00100100, //   *  *
    0b00100100, //   *  *
    0b01111110, //  ******
    0b01000010, //  *    *
    0b01000010, //  *    *
    0b01000010, //  *    *
    0b11100111, // ***  ***
    0b00000000,
];
