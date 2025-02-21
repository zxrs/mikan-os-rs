use crate::{Result, x86};

pub const CONFIG_ADDRESS: u16 = 0x0cf8;
pub const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Default, Clone, Copy)]
pub struct Device {
    bus: u8,
    device: u8,
    function: u8,
    header_thype: u8,
}

pub static mut DEVICES: [Option<Device>; 32] = [None; 32];
pub static mut NUM_DEVICES: usize = 0;

pub fn scan_all_bus() -> Result<()> {
    Ok(())
}

pub fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    let shl = |x: u32, bits: usize| x << bits;

    shl(1, 31)
        | shl(bus as u32, 16)
        | shl(device as u32, 11)
        | shl(function as u32, 8)
        | (reg_addr as u32 & 0xfc)
}

pub fn write_address(address: u32) {
    x86::io_out32(CONFIG_ADDRESS, address);
}

pub fn write_data(value: u32) {
    x86::io_out32(CONFIG_DATA, value);
}

pub fn read_data() -> u32 {
    x86::io_in32(CONFIG_DATA)
}

pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data() as _
}
