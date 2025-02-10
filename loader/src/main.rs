#![no_std]
#![no_main]

use core::fmt::Write;
use core::slice;

mod x86;

mod uefi;
use uefi::{
    EFIEventType, EFIGraphicsOutputProtocol, EFIHandle, EFISimpleTextOutputProtocolWriter,
    EFISystemTable, EFITimerDelay, EFITpl, MemoryDescriptorVisitor,
};

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
    ($arg:expr $(,)?) => {{
        println!(
            "[{}:{}] {} = {:#?}",
            file!(),
            line!(),
            stringify!($arg),
            $arg
        );
    }};
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
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

    let event = system_table
        .boot_services
        .create_event(EFIEventType(EFIEventType::TIMER), EFITpl(EFITpl::CALLBACK))
        .unwrap();
    let events = [event];
    (0..30).for_each(|i| {
        system_table
            .boot_services
            .set_timer(event, EFITimerDelay::Relative, 10_000_000)
            .unwrap();
        system_table.boot_services.wait_for_event(&events).unwrap();
        frame_buffer.chunks_exact_mut(4).for_each(|c| {
            c[0] = 0;
            c[1] = 0;
            c[2] = 0;
            c[i % 3] = 255;
        });
    });

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
