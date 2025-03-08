use core::arch::asm;

pub fn halt() {
    unsafe { asm!("hlt") };
}

pub fn io_out32(addr: u16, data: u32) {
    unsafe { asm!("out dx, eax", in("dx") addr, in("eax") data) };
}

pub fn io_in32(addr: u16) -> u32 {
    let a;
    unsafe { asm!("in eax, dx", in("dx") addr, out("eax") a) };
    a
}

pub fn get_cs() -> u16 {
    let a;
    unsafe { asm!("mov {0:x}, cs", out(reg) a) };
    a
}

#[repr(C, packed)]
pub struct IdtParam {
    pub limit: u16,
    pub base: usize,
}

pub fn load_idt(param: &IdtParam) {
    unsafe { asm!("lidt [rcx]", in("rcx") param) };
}
