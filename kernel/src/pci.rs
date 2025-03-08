#![allow(unused)]

use bit_field::BitField;

use crate::{Result, x86};

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MSITriggerMode {
    Edge,
    Level,
}

#[derive(Debug, Clone, Copy)]
pub struct MSIDeliveryMode(u8);

#[allow(non_snake_case)]
impl MSIDeliveryMode {
    const FIXED: u8 = 0b000;
    const LOWEST_PRIORITY: u8 = 0b001;
    const SMI: u8 = 0b010;
    const NMI: u8 = 0b100;
    const INIT: u8 = 0b101;
    const EXTINT: u8 = 0b111;

    pub fn Fixed() -> Self {
        Self(Self::FIXED)
    }

    pub fn LowestPriority() -> Self {
        Self(Self::LOWEST_PRIORITY)
    }

    pub fn Smi() -> Self {
        Self(Self::SMI)
    }

    pub fn Nmi() -> Self {
        Self(Self::NMI)
    }

    pub fn Init() -> Self {
        Self(Self::INIT)
    }

    pub fn ExtInt() -> Self {
        Self(Self::EXTINT)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub class_code: (u8, u8, u8),
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct MSICapability {
    header: MSICapabilityHeader,
    msg_addr: u32,
    msg_upper_addr: u32,
    msg_data: u32,
    mask_bits: u32,
    pending_bits: u32,
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy)]
pub struct MSICapabilityHeader(u32);

impl MSICapabilityHeader {
    fn data(&self) -> u32 {
        self.0
    }

    fn cap_id(&self) -> u8 {
        self.0.get_bits(0..8) as u8
    }

    fn set_msi_enable(&mut self, value: bool) {
        self.0.set_bit(16, value);
    }

    fn addr_64_capable(&self) -> u8 {
        self.0.get_bit(23) as u8
    }

    fn per_vector_mask_capable(&self) -> bool {
        self.0.get_bit(24)
    }

    fn multi_msg_capable(&self) -> u8 {
        self.0.get_bits(17..20) as u8
    }

    fn set_multi_msg_enable(&mut self, value: u8) {
        self.0.set_bits(17..20, value as u32);
    }
}

pub static mut DEVICES: [Device; 32] = unsafe { core::mem::zeroed() };

pub fn scan_all_bus() -> Result<()> {
    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return scan_bus(0);
    }
    (1..8)
        .filter(|function| read_vendor_id(0, 0, *function) != 0xffff)
        .try_for_each(|function| scan_bus(function))
}

fn scan_bus(bus: u8) -> Result<()> {
    (0..32)
        .filter(|device| read_vendor_id(bus, *device, 0) != 0xffff)
        .try_for_each(|device| scan_device(bus, device))
}

fn read_msi_capability(dev: &Device, cap_addr: u8) -> MSICapability {
    let mut msi_cap = MSICapability::default();
    msi_cap.header = MSICapabilityHeader(read_conf_reg(dev, cap_addr));
    msi_cap.msg_addr = read_conf_reg(dev, cap_addr + 4);

    let mut msg_data_addr = cap_addr + 8;
    if msi_cap.header.addr_64_capable() > 0 {
        msi_cap.msg_upper_addr = read_conf_reg(dev, cap_addr + 8);
        msg_data_addr = cap_addr + 12;
    }

    msi_cap.msg_data = read_conf_reg(dev, msg_data_addr);

    if msi_cap.header.per_vector_mask_capable() > 0 {
        msi_cap.mask_bits = read_conf_reg(dev, msg_data_addr + 4);
        msi_cap.pending_bits = read_conf_reg(dev, msg_data_addr + 8);
    }

    msi_cap
}

fn write_msi_capability(dev: &Device, cap_addr: u8, msi_cap: &MSICapability) {
    write_conf_reg(dev, cap_addr, msi_cap.header.data());
    write_conf_reg(dev, cap_addr + 4, msi_cap.msg_addr);

    let mut msg_data_addr = cap_addr + 8;
    if msi_cap.header.addr_64_capable() > 0 {
        write_conf_reg(dev, cap_addr + 8, msi_cap.msg_upper_addr);
        msg_data_addr = cap_addr + 12;
    }

    write_conf_reg(dev, msg_data_addr, msi_cap.msg_data);

    if msi_cap.header.per_vector_mask_capable() > 0 {
        write_conf_reg(dev, msg_data_addr + 4, msi_cap.mask_bits);
        write_conf_reg(dev, msg_data_addr + 8, msi_cap.pending_bits);
    }
}

fn configure_msi_resiter(
    dev: &Device,
    cap_addr: u8,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u32,
) -> Result<()> {
    let mut msi_cap = read_msi_capability(dev, cap_addr);

    if msi_cap.header.multi_msg_capable() as u32 <= num_vector_exponent {
        msi_cap
            .header
            .set_multi_msg_enable(msi_cap.header.multi_msg_capable());
    } else {
        msi_cap
            .header
            .set_multi_msg_enable(num_vector_exponent as _);
    }

    msi_cap.header.set_msi_enable(1);
    msi_cap.msg_addr = msg_addr;
    msi_cap.msg_data = msg_data;

    write_msi_capability(dev, cap_addr, &msi_cap);
    Ok(())
}

#[repr(transparent)]
#[derive(Debug, Default)]
struct CapabilityHeader(u32);

impl CapabilityHeader {
    const CAP_ID_MASK: u32 = 0b0000_0000_0000_0000_0000_0000_1111_1111;
    const NEXT_PTR_MASK: u32 = 0b0000_0000_0000_0000_1111_1111_0000_0000;

    fn cap_id(&self) -> u8 {
        (self.0 | Self::CAP_ID_MASK) as _
    }

    fn next_ptr(&self) -> u8 {
        (self.0 | Self::NEXT_PTR_MASK) as _
    }
}

fn read_capability_header(dev: &Device, addr: u8) -> CapabilityHeader {
    CapabilityHeader(read_conf_reg(dev, addr))
}

const CAPABILITY_MSI: u8 = 0x05;
const CAPABILITY_MSIX: u8 = 0x11;

fn configure_msi(
    dev: &Device,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u32,
) -> Result<()> {
    let mut cap_addr = (read_conf_reg(dev, 0x34) & 0xff) as u8;
    let mut msi_cap_addr = 0;
    let mut msix_cap_addr = 0;
    while cap_addr != 0 {
        let header = read_capability_header(dev, cap_addr);
        if header.cap_id() == CAPABILITY_MSI {
            msi_cap_addr = cap_addr;
        } else if header.cap_id() == CAPABILITY_MSIX {
            msix_cap_addr = cap_addr;
        }
        cap_addr = header.next_ptr();
    }
    if msi_cap_addr > 0 {
        return configure_msi_resiter(dev, msi_cap_addr, msg_addr, msg_data, num_vector_exponent);
    } else if msix_cap_addr > 0 {
        return Err("not implemented.");
    }
    Err("no pci msi.")
}

pub fn configure_msi_fixed_destication(
    dev: &Device,
    apic_id: u8,
    trigger_mode: MSITriggerMode,
    delivery_mode: MSIDeliveryMode,
    vector: u8,
    num_vector_exponent: u32,
) -> Result<()> {
    let msg_addr = 0xfee0000 | ((apic_id as u32) << 12);
    let mut msg_data = ((delivery_mode.0 as u32) << 8) | vector as u32;
    if trigger_mode == MSITriggerMode::Level {
        msg_data |= 0xc000;
    }
    configure_msi(dev, msg_addr, msg_data, num_vector_exponent)?;
    Ok(())
}

fn scan_function(bus: u8, device: u8, function: u8) -> Result<()> {
    let header_type = read_header_type(bus, device, function);
    let class_code = read_class_code(bus, device, function);

    let dev = Device {
        bus,
        device,
        function,
        header_type,
        class_code,
    };

    add_device(dev)?;

    if class_code.0 == 0x06 && class_code.1 == 0x04 {
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus = (bus_numbers >> 8) as u8;
        return scan_bus(secondary_bus);
    }
    Ok(())
}

fn scan_device(bus: u8, device: u8) -> Result<()> {
    scan_function(bus, device, 0)?;

    if is_single_function_device(read_header_type(bus, device, 0)) {
        return Ok(());
    }

    (0..8)
        .filter(|function| read_vendor_id(bus, device, *function) != 0xffff)
        .try_for_each(|function| scan_function(bus, device, function))
}

fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    let shl = |x: u32, bits: usize| x << bits;

    shl(1, 31)
        | shl(bus as u32, 16)
        | shl(device as u32, 11)
        | shl(function as u32, 8)
        | (reg_addr as u32 & 0xfc)
}

pub fn num_devices() -> usize {
    unsafe { DEVICES }.iter().filter(|d| d.is_some()).count()
}

fn add_device(device: Device) -> Result<()> {
    if num_devices() == unsafe { DEVICES }.len() {
        return Err("full");
    }

    unsafe {
        DEVICES[num_devices()] = Some(device);
    };
    Ok(())
}

fn write_address(address: u32) {
    x86::io_out32(CONFIG_ADDRESS, address);
}

fn write_data(value: u32) {
    x86::io_out32(CONFIG_DATA, value);
}

fn read_data() -> u32 {
    x86::io_in32(CONFIG_DATA)
}

pub fn read_vendor_id_from_device(dev: &Device) -> u16 {
    read_device_id(dev.bus, dev.device, dev.function)
}

fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data() as _
}

fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    (read_data() >> 16) as _
}

fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    (read_data() >> 16) as _
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> (u8, u8, u8) {
    write_address(make_address(bus, device, function, 0x08));
    let reg = read_data();
    ((reg >> 24) as _, (reg >> 16) as _, (reg >> 8) as _)
}

fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

fn read_conf_reg(dev: &Device, reg_addr: u8) -> u32 {
    write_address(make_address(dev.bus, dev.device, dev.function, reg_addr));
    read_data()
}

fn write_conf_reg(dev: &Device, reg_addr: u8, value: u32) {
    write_address(make_address(dev.bus, dev.device, dev.function, reg_addr));
    write_data(value)
}

fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

pub fn read_bar(device: &Device, bar_index: u32) -> Result<u64> {
    if bar_index >= 6 {
        return Err("index is out of range.");
    }

    let addr = calc_bar_address(bar_index);
    let bar = read_conf_reg(device, addr);

    if (bar & 4) == 0 {
        return Ok(bar as _);
    }

    if bar_index >= 5 {
        return Err("index of out of range");
    }

    let bar_upper = read_conf_reg(device, addr + 4);
    Ok(bar as u64 | ((bar_upper as u64) << 32))
}

fn calc_bar_address(bar_index: u32) -> u8 {
    0x10u8 + 4 * (bar_index as u8)
}
