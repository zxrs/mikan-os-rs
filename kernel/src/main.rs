#![no_std]
#![no_main]

use core::arch::asm;
use core::slice;

mod frame_buffer;
use frame_buffer::{BGRPixelWriter, FrameBufferConfig, PixelFormat, PixelWriter, Rgb};

pub type Result<T> = core::result::Result<T, &'static str>;

#[unsafe(no_mangle)]
extern "C" fn kernel_main(frame_buffer_config: &mut FrameBufferConfig) -> ! {
    let v = frame_buffer_config.vertical_resolution;
    let h = frame_buffer_config.horizontal_resolution;
    let mut writer = match frame_buffer_config.pixel_format {
        PixelFormat::BGRR => BGRPixelWriter::new(frame_buffer_config),
        _ => unimplemented!(),
    };
    (0..v).for_each(|y| {
        (0..h).for_each(|x| {
            writer.write(x, y, Rgb::new(255, 255, 255)).unwrap();
        });
    });
    // frame_buffer.fill(255);
    // let rgb = Rgb::new(0, 255, 0);
    // (0..200).for_each(|y| {
    //     (0..100).for_each(|x| {
    //         frame_buffer_config.write_pixel(x, y, rgb).unwrap();
    //     });
    // });
    // let frame_buffer = unsafe { slice::from_raw_parts_mut(frame_buffer_base, frame_buffer_size) };
    // frame_buffer.chunks_exact_mut(4).for_each(|c| {
    //     c[0] = 255;
    //     c[1] = 0;
    //     c[2] = 0;
    // });
    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic_impl(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
