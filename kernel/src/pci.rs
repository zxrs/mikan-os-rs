use crate::{Result, x86};

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Default, Clone, Copy)]
struct Device {
    bus: u8,
    device: u8,
    function: u8,
    header_type: u8,
}

static mut DEVICES: [Option<Device>; 32] = [None; 32];

fn scan_all_bus() -> Result<()> {
    Ok(())
}

fn scan_bus(bus: u8) -> Result<()> {
    (0..32)
        .filter(|device| read_vendor_id(bus, *device, 0) != 0xffff)
        .try_for_each(|device| scan_device(bus, device))
}

fn scan_function(bus: u8, device: u8, function: u8) -> Result<()> {
    let header_type = read_header_type(bus, device, function);
    add_device(bus, device, function, header_type)?;

    let class_code = read_class_code(bus, device, function);
    let base = (class_code >> 24) as u8;
    let sub = (class_code >> 16) as u8;

    if base == 0x06 && sub == 0x04 {
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus = (bus_numbers >> 8) as u8;
        return scan_bus(secondary_bus);
    }
    Ok(())
}

fn scan_device(bus: u8, device: u8) -> Result<()> {
    scan_function(bus, device, 0);

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

fn num_devices() -> usize {
    unsafe { DEVICES }.iter().filter(|d| d.is_some()).count()
}

fn add_device(bus: u8, device: u8, function: u8, header_type: u8) -> Result<()> {
    if num_devices() == unsafe { DEVICES }.len() {
        return Err("full");
    }

    unsafe {
        DEVICES[num_devices()] = Some(Device {
            bus,
            device,
            function,
            header_type,
        })
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

fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x08));
    read_data()
}

fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}
