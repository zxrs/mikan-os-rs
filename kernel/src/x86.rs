use core::arch::asm;

pub fn halt() {
    unsafe { asm!("hlt") };
}

pub fn io_out32(addr: u16, data: u32) {
    unsafe { asm!("out {0:x}, eax", in(reg) addr, in("eax") data) };
}

pub fn io_in32(addr: u16) -> u32 {
    let a;
    unsafe { asm!("in eax {0:x}", in(reg) addr, out("eax") a) };
    a
}
