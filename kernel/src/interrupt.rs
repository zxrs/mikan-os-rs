use bit_field::BitField;

pub static mut IDT: [InterruptDescriptor; 256] = unsafe { core::mem::zeroed() };

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct InterruptDescriptor {
    offset_low: u16,
    segment_selector: u16,
    attr: InterruptDescriptorAttribute,
    offset_middle: u16,
    offset_high: u32,
    _reserved: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct InterruptVector(usize);

impl InterruptVector {
    const XHCI: usize = 0x40;

    #[allow(non_snake_case)]
    pub fn Xhci() -> Self {
        Self(Self::XHCI)
    }
}

impl From<usize> for InterruptVector {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl InterruptVector {
    pub fn get(&self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DescriptorType(u16);

#[allow(non_snake_case)]
#[allow(unused)]
impl DescriptorType {
    const UPPER_BYTES: u16 = 0;
    const LDT: u16 = 2;
    const TSS_AVAILABLE: u16 = 9;
    const TSS_BUSY: u16 = 11;
    const CALL_GATE: u16 = 12;
    const INTERRUPT_GATE: u16 = 14;
    const TRAP_GATE: u16 = 15;

    pub fn UpperBytes() -> Self {
        Self(Self::UPPER_BYTES)
    }

    pub fn Ldt() -> Self {
        Self(Self::LDT)
    }

    pub fn TssAvailable() -> Self {
        Self(Self::TSS_AVAILABLE)
    }

    pub fn TssBusy() -> Self {
        Self(Self::TSS_BUSY)
    }

    pub fn CallGate() -> Self {
        Self(Self::CALL_GATE)
    }

    pub fn InterruptGate() -> Self {
        Self(Self::INTERRUPT_GATE)
    }

    pub fn TrapGate() -> Self {
        Self(Self::TRAP_GATE)
    }
}

impl From<u16> for DescriptorType {
    fn from(value: u16) -> Self {
        DescriptorType(value)
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy)]
pub struct InterruptDescriptorAttribute(u16);

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
    let p = 0xfee000b0 as *mut u32;
    unsafe { p.write_volatile(0) };
}

pub fn set_idt_entry(
    desc: &mut InterruptDescriptor,
    attr: InterruptDescriptorAttribute,
    offset: u64,
    segment_selector: u16,
) {
    desc.attr = attr;
    desc.offset_low = (offset & 0xffff) as _;
    desc.offset_middle = ((offset >> 16) & 0xffff) as _;
    desc.offset_high = (offset >> 32) as _;
    desc.segment_selector = segment_selector;
}

pub fn make_idt_attr(
    descriptor_type: DescriptorType,
    descriptor_privilage_level: u16,
    present: bool,
    interrupt_stack_table: u16,
) -> InterruptDescriptorAttribute {
    let mut field = 0;
    field
        .set_bit(15, present)
        .set_bits(13..15, descriptor_privilage_level)
        .set_bits(8..12, descriptor_type.0)
        .set_bits(0..3, interrupt_stack_table);
    InterruptDescriptorAttribute(field)
}
