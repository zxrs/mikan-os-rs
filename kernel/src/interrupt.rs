use core::ptr;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

pub fn notify_end_of_interrupt() {
    unsafe { ptr::null_mut::<u32>().offset(0xfee000b0).write_volatile(0) };
}
