use crate::{DescriptorType, x86};
use bit_field::BitField;

static mut GDT: [SegmentDescriptor; 3] = unsafe { core::mem::zeroed() };
fn gdt() -> &'static mut [SegmentDescriptor] {
    #[allow(static_mut_refs)]
    unsafe {
        &mut GDT
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct SegmentDescriptor(u64);

impl SegmentDescriptor {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn set_data(&mut self, value: u64) {
        self.0 = value;
    }

    pub fn set_base_high(&mut self, value: u8) {
        self.0.set_bits(..8, value as _);
    }

    pub fn set_granularity(&mut self, value: bool) {
        self.0.set_bit(8, value);
    }

    pub fn set_default_operation_size(&mut self, value: bool) {
        self.0.set_bit(9, value);
    }

    pub fn set_long_mode(&mut self, value: bool) {
        self.0.set_bit(10, value);
    }

    pub fn set_available(&mut self, value: bool) {
        self.0.set_bit(11, value);
    }

    pub fn set_limit_high(&mut self, value: u8) {
        self.0.set_bits(12..16, value as _);
    }

    pub fn set_present(&mut self, value: bool) {
        self.0.set_bit(16, value);
    }

    pub fn set_descriptor_privilege_level(&mut self, value: u8) {
        self.0.set_bits(17..19, value as _);
    }

    pub fn set_system_segment(&mut self, value: bool) {
        self.0.set_bit(19, value);
    }

    pub fn set_type(&mut self, value: DescriptorType) {
        self.0.set_bits(20..24, value.to_u16() as _);
    }

    pub fn set_base_middle(&mut self, value: u8) {
        self.0.set_bits(24..32, value as _);
    }

    pub fn set_base_low(&mut self, value: u16) {
        self.0.set_bits(32..48, value as _);
    }

    pub fn set_limit_low(&mut self, value: u16) {
        self.0.set_bits(48..64, value as _);
    }
}

pub fn setup_segments() {
    gdt()[0].set_data(0);
    set_code_segment(&mut gdt()[1], DescriptorType::ExecuteRead(), 0, 0, 0xfffff);
    set_data_segment(&mut gdt()[2], DescriptorType::ReadWrite(), 0, 0, 0xfffff);
    let param = x86::GdtParam {
        limit: (size_of::<[SegmentDescriptor; 3]>() - 1) as u16,
        base: gdt() as *mut [SegmentDescriptor] as *mut SegmentDescriptor as usize,
    };
    x86::load_gdt(&param);
}

fn set_code_segment(
    desc: &mut SegmentDescriptor,
    typ: DescriptorType,
    descriptor_privilege_level: u32,
    base: u32,
    limit: u32,
) {
    desc.set_data(0);
    desc.set_base_low((base & 0xffff) as _);
    desc.set_base_middle(((base >> 16) & 0xff) as _);
    desc.set_base_high(((base >> 24) & 0xff) as _);

    desc.set_limit_low((limit & 0xffff) as _);
    desc.set_limit_high(((limit >> 16) & 0xf) as _);

    desc.set_type(typ);
    desc.set_system_segment(true);
    desc.set_descriptor_privilege_level(descriptor_privilege_level as _);
    desc.set_present(true);
    desc.set_available(false);
    desc.set_long_mode(true);
    desc.set_default_operation_size(false);
    desc.set_granularity(true);
}

fn set_data_segment(
    desc: &mut SegmentDescriptor,
    typ: DescriptorType,
    descriptor_privilege_level: u32,
    base: u32,
    limit: u32,
) {
    set_code_segment(desc, typ, descriptor_privilege_level, base, limit);
    desc.set_long_mode(false);
    desc.set_default_operation_size(true);
}
