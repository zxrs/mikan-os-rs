#![no_std]
#![no_main]

use core::arch::asm;

mod frame_buffer;
use frame_buffer::{BGRPixelWriter, FrameBufferConfig, PixelFormat, PixelWriter, Rgb};

mod font;
use font::write_ascii;

pub type Result<T> = core::result::Result<T, &'static str>;

#[unsafe(no_mangle)]
extern "C" fn kernel_main(frame_buffer_config: &mut FrameBufferConfig) -> ! {
    frame_buffer_config.as_slice_mut().fill(255);
    let mut writer = match frame_buffer_config.pixel_format {
        PixelFormat::BGRR => BGRPixelWriter::new(frame_buffer_config),
        _ => unimplemented!(),
    };
    (0..100).for_each(|y| {
        (0..200).for_each(|x| {
            writer.write(x, y, Rgb::new(0, 0, 255)).unwrap();
        });
    });
    write_ascii(&mut writer, 50, 50, 'A', Rgb::white()).unwrap();
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
