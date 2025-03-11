use core::arch::asm;

unsafe extern "C" {
    fn SetCSSS(cs: u16, ss: u16);
}

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

pub fn sti() {
    unsafe { asm!("sti") };
}

pub fn cli() {
    unsafe { asm!("cli") };
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

pub fn switch_rsp(new_rsp: usize, kernel_main_new_stack: fn() -> !) {
    unsafe {
        asm!(
            "mov rsp, {0:r}",
            "jmp {1:r}",
            in(reg) new_rsp,
            in(reg) kernel_main_new_stack
        );
    }
}

#[repr(C, packed)]
pub struct GdtParam {
    pub limit: u16,
    pub base: usize,
}

pub fn load_gdt(param: &GdtParam) {
    unsafe { asm!("lgdt [rcx]", in("rcx") param) };
}

pub fn set_ds_all(value: u16) {
    unsafe {
        asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov fs, {0:x}",
            "mov gs, {0:x}",
            in(reg) value,
        )
    };
}

pub fn set_cs_ss(cs: u16, ss: u16) {
    unsafe { SetCSSS(cs, ss) };
}

pub fn set_cr3(value: u64) {
    unsafe { asm!("mov cr3, {0:r}", in(reg) value) };
}
