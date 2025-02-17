use crate::Result;
use core::slice;

#[allow(unused)]
pub struct FrameBufferConfig {
    frame_buffer: *mut u8,
    pixels_per_scan_line: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: PixelFormat,
}

impl FrameBufferConfig {
    pub fn frame_buffer(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.frame_buffer,
                (self.horizontal_resolution * self.vertical_resolution * 4) as usize,
            )
        }
    }
}

#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    RGBR, // red. green, blue, reserved
    BGRR, // blue, greem, red, reserved
}

pub trait PixelWriter {
    fn pixel_at(&mut self, x: u32, y: u32) -> Option<&mut [u8]>;
    fn write(&mut self, x: u32, y: u32, rgb: Rgb) -> Result<()>;
}

pub struct BGRPixelWriter(&'static mut FrameBufferConfig);

impl BGRPixelWriter {
    pub fn new(frame_buffer_config: &'static mut FrameBufferConfig) -> Self {
        Self(frame_buffer_config)
    }
}

impl PixelWriter for BGRPixelWriter {
    fn pixel_at(&mut self, x: u32, y: u32) -> Option<&mut [u8]> {
        let index = (y * self.0.horizontal_resolution + x) as usize;
        self.0.frame_buffer().chunks_mut(4).nth(index)
    }

    fn write(&mut self, x: u32, y: u32, rgb: Rgb) -> Result<()> {
        let pixel = self.pixel_at(x, y).ok_or("out of buffer")?;
        pixel[0] = rgb.b;
        pixel[1] = rgb.g;
        pixel[2] = rgb.r;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[allow(unused)]
impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }

    pub fn black() -> Self {
        Default::default()
    }

    pub fn red() -> Self {
        Self {
            r: 255,
            ..Default::default()
        }
    }

    pub fn green() -> Self {
        Self {
            g: 255,
            ..Default::default()
        }
    }

    pub fn blue() -> Self {
        Self {
            b: 255,
            ..Default::default()
        }
    }

    pub fn yellow() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 0,
        }
    }

    pub fn purple() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
        }
    }

    pub fn cyan() -> Self {
        Self {
            r: 0,
            g: 255,
            b: 255,
        }
    }
}
