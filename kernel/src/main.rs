#![no_std]
#![no_main]

use core::arch::asm;

#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
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
