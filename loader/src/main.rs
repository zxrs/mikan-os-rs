#![allow(internal_features)]
#![feature(core_intrinsics)]
#![no_std]
#![no_main]

use core::fmt::Write;
use core::slice;
use utf16_lit::utf16_null as w;

static mut WRITER: Option<EFISimpleTextOutputProtocolWriter<'_>> = None;

#[macro_use]
mod macros;
//mod cube;
mod elf;
use elf::{Elf64_Ehdr, calc_load_address_range, copy_load_segments};
mod uefi;
mod x86;
use uefi::{
    CChar, EFIAllocateType, EFIFileInfo, EFIGraphicsOutputProtocol, EFIHandle,
    EFILoadedImageProtocol, EFIMemoryType, EFISimpleFileSystemProtocol,
    EFISimpleTextOutputProtocolWriter, EFISystemTable, FileAttributes, FileMode,
    MemoryDescriptorVisitor,
};

/// [4.1.1. EFI_IMAGE_ENTRY_POINT](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#efi-image-entry-point)
#[unsafe(no_mangle)]
fn efi_main(image_handle: EFIHandle, system_table: &'static EFISystemTable) -> ! {
    system_table.con_out.clear_screen();
    unsafe { WRITER = Some(EFISimpleTextOutputProtocolWriter::new(system_table.con_out)) };
    println!("{}", system_table.firmware_vendor);

    let memory_map = system_table.boot_services.get_memory_map().unwrap();
    dbg!(memory_map.size);

    let visitor = MemoryDescriptorVisitor::new(&memory_map);
    //visitor.take(3).for_each(|d| dbg!(d));
    dbg!(visitor.count());

    let graphics_output = system_table
        .boot_services
        .locate_protocol::<EFIGraphicsOutputProtocol>()
        .unwrap();
    dbg!(graphics_output.mode.frame_buffer_base);
    let frame_buffer = unsafe {
        slice::from_raw_parts_mut(
            graphics_output.mode.frame_buffer_base as *mut u8,
            graphics_output.mode.frame_buffer_size as _,
        )
    };
    dbg!(frame_buffer.len());
    dbg!(graphics_output.mode.info.vertical_resolution);
    dbg!(graphics_output.mode.info.horizontal_resolution);
    dbg!(graphics_output.mode.info.pixels_per_scan_line);

    let loaded_image = system_table
        .boot_services
        .open_protocol::<EFILoadedImageProtocol>(image_handle, image_handle)
        .unwrap();

    let fs = system_table
        .boot_services
        .open_protocol::<EFISimpleFileSystemProtocol>(loaded_image.device_handle(), image_handle)
        .unwrap();

    let root_dir = fs.open_volume().unwrap();

    let kernel_file = root_dir
        .open(
            CChar::from_ptr(w!("kernel.elf").as_ptr()),
            FileMode::Read(),
            FileAttributes::default(),
        )
        .unwrap();

    let mut file_info_buffer = [0; size_of::<EFIFileInfo>() + size_of::<u16>() * 12];
    let file_info = kernel_file
        .get_info::<EFIFileInfo>(&mut file_info_buffer)
        .unwrap();
    let kernel_file_size = file_info.file_size;
    dbg!(kernel_file_size);

    let kernel_buffer = system_table
        .boot_services
        .allocate_pool(EFIMemoryType::LoaderData, kernel_file_size as usize)
        .unwrap();

    kernel_file
        .read(kernel_file_size as usize, kernel_buffer.as_ptr().addr())
        .unwrap();

    let kernel_ehdr = unsafe { &*(kernel_buffer.as_ptr() as *const Elf64_Ehdr) };

    let (first, last) = calc_load_address_range(kernel_ehdr);

    let num_pages = (last - first).div_ceil(0x1000);

    let kernel_base_address = system_table
        .boot_services
        .allocate_pages(
            EFIAllocateType::AllocateAddress,
            EFIMemoryType::LoaderData,
            num_pages as usize,
            first,
        )
        .unwrap();

    println!("kernel_base_address: 0x{:08x?}", kernel_base_address);

    copy_load_segments(system_table, kernel_ehdr).unwrap();

    dbg!(kernel_ehdr.entry_addr());

    dbg!(graphics_output.mode.info.pixels_per_scan_line);

    if system_table
        .boot_services
        .exit_boot_services(image_handle, memory_map.map_key)
        .is_err()
    {
        let memory_map = system_table.boot_services.get_memory_map().unwrap();
        system_table
            .boot_services
            .exit_boot_services(image_handle, memory_map.map_key)
            .unwrap();
    }

    #[allow(dead_code)]
    struct FrameBufferConfig {
        frame_buffer: *mut u8,
        pixels_per_scan_line: u32,
        horizontal_resolution: u32,
        vertical_resolution: u32,
        pixel_format: PixelFormat,
    }

    #[allow(clippy::upper_case_acronyms)]
    enum PixelFormat {
        RGBR, // red. green, blue, reserved
        BGRR, // blue, greem, red, reserved
    }

    let config = FrameBufferConfig {
        frame_buffer: graphics_output.mode.frame_buffer_base as _,
        pixels_per_scan_line: graphics_output.mode.info.pixels_per_scan_line,
        horizontal_resolution: graphics_output.mode.info.horizontal_resolution,
        vertical_resolution: graphics_output.mode.info.vertical_resolution,
        pixel_format: match graphics_output.mode.info.pixel_format {
            0 => PixelFormat::RGBR,
            1 => PixelFormat::BGRR,
            _ => unimplemented!(),
        },
    };

    let entry_point = unsafe {
        core::mem::transmute::<*const u8, extern "sysv64" fn(&FrameBufferConfig)>(
            kernel_ehdr.entry_addr() as *const u8,
        )
    };
    entry_point(&config);

    // cube::rotate(system_table, frame_buffer);

    unreachable!();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC!!!!!");
    println!("{info}");
    loop {
        x86::halt();
    }
}
