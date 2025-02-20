use core::arch::asm;

pub fn halt() {
    unsafe { asm!("hlt") };
}

pub fn io_out32(addr: u16, data: u32) -> u32 {
    let a;
    unsafe { asm!("out {0:l}, eax", in(reg) addr, inlateout("eax") data => a ) };
    a
}
