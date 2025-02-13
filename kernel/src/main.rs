#![no_std]
#![no_main]

use core::arch::asm;
use core::slice;

#[unsafe(no_mangle)]
extern "C" fn kernel_main(frame_buffer_base: *mut u8, frame_buffer_size: usize) -> ! {
    let frame_buffer = unsafe { slice::from_raw_parts_mut(frame_buffer_base, frame_buffer_size) };
    frame_buffer.chunks_exact_mut(4).for_each(|c| {
        c[0] = 255;
        c[1] = 0;
        c[2] = 0;
    });
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
