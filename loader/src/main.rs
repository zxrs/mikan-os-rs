#![no_std]
#![no_main]

use core::fmt::Write;

mod x86;

mod uefi;
use uefi::{EFIHandle, EFISimpleTextOutputProtocolWriter, EFISystemTable};

/// [4.1.1. EFI_IMAGE_ENTRY_POINT](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#efi-image-entry-point)
#[unsafe(no_mangle)]
fn efi_main<'a>(_: EFIHandle, system_table: &'a EFISystemTable) -> ! {
    system_table.con_out.clear_screen();
    let mut writer = EFISimpleTextOutputProtocolWriter::new(system_table.con_out);
    writeln!(&mut writer, "Hello, Foo!").unwrap();
    writeln!(&mut writer, "{}", system_table.firmware_vendor).unwrap();
    loop {
        x86::halt();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        x86::halt();
    }
}
