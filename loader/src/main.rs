#![allow(internal_features)]
#![feature(core_intrinsics)]
#![no_std]
#![no_main]

use core::fmt::Write;
use core::slice;

static mut WRITER: Option<EFISimpleTextOutputProtocolWriter<'_>> = None;

#[macro_use]
mod macros;

mod cube;

mod x86;

mod uefi;
use uefi::{
    EFIEventType, EFIGraphicsOutputProtocol, EFIHandle, EFILoadedImageProtocol,
    EFISimpleFileSystemProtocol, EFISimpleTextOutputProtocolWriter, EFISystemTable, EFITimerDelay,
    EFITpl, MemoryDescriptorVisitor,
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

    // cube::rotate(system_table, frame_buffer);

    //let event = system_table
    //    .boot_services
    //    .create_event(EFIEventType::Timer(), EFITpl::Callback())
    //    .unwrap();
    //let events = [event];
    //(0..30).for_each(|i| {
    //    system_table
    //        .boot_services
    //        .set_timer(event, EFITimerDelay::Relative, 10_000_000)
    //        .unwrap();
    //    system_table.boot_services.wait_for_event(&events).unwrap();
    //    frame_buffer.chunks_exact_mut(4).for_each(|c| {
    //        c[0] = 0;
    //        c[1] = 0;
    //        c[2] = 0;
    //        c[i % 3] = 255;
    //    });
    //});

    loop {
        x86::halt();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC!!!!!");
    println!("{info}");
    loop {
        x86::halt();
    }
}
