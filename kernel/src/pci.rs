use crate::{Result, x86};

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Default, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub class_code: (u8, u8, u8),
}

pub static mut DEVICES: [Option<Device>; 32] = [None; 32];

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

    if (bar_index >= 5) {
        return Err("index of out of range");
    }

    let bar_upper = read_conf_reg(device, addr + 4);
    Ok(bar as u64 | (bar_upper as u64) << 32)
}

fn calc_bar_address(bar_index: u32) -> u8 {
    0x10u8 + 4 * (bar_index as u8)
}
