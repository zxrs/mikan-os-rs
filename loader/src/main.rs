#![no_std]
#![no_main]

use core::fmt::Write;

mod x86;

mod uefi;
use uefi::{EFIHandle, EFISimpleTextOutputProtocolWriter, EFISystemTable, MemoryDescriptorVisitor};

static mut WRITER: Option<EFISimpleTextOutputProtocolWriter<'_>> = None;

macro_rules! print {
    ($($args:tt)*) => {{
        #[allow(static_mut_refs)]
        if let Some(writer) = unsafe { WRITER.as_mut() } {
            write!(writer, "{}", format_args!($($args)*)).unwrap();
        }
    }};
}

macro_rules! println {
    ($($args:tt)*) => {{
        print!("{}\n", format_args!($($args)*));
    }};
}

macro_rules! dbg {
    () => {{
        println!("[{}:{}]", file!(), line!());
    }};
    ($arg:expr) => {{
        println!(
            "[{}:{}] {} = {:#?}",
            file!(),
            line!(),
            stringify!($arg),
            $arg
        );
    }};
}

/// [4.1.1. EFI_IMAGE_ENTRY_POINT](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#efi-image-entry-point)
#[unsafe(no_mangle)]
fn efi_main(_: EFIHandle, system_table: &'static EFISystemTable) -> ! {
    system_table.con_out.clear_screen();
    unsafe { WRITER = Some(EFISimpleTextOutputProtocolWriter::new(system_table.con_out)) };
    println!("{}", system_table.firmware_vendor);

    let memory_map = system_table.boot_services.get_memory_map().unwrap();
    dbg!(memory_map.size);

    let visitor = MemoryDescriptorVisitor::new(&memory_map);
    //visitor.take(3).for_each(|d| dbg!(d));
    dbg!(visitor.count());

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
