use core::slice;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FrameBufferConfig {
    pub frame_buffer: *mut u8,
    pub pixels_per_scan_line: u32,
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

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    RGBR, // red. green, blue, reserved
    BGRR, // blue, greem, red, reserved
}
