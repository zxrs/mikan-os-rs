#![feature(str_from_raw_parts)]
#![no_std]
#![no_main]

use core::arch::asm;
use core::fmt::Write;

#[macro_use]
mod macros;

mod fonts;

mod console;
use console::Console;

mod frame_buffer;
use frame_buffer::{BGRPixelWriter, FrameBufferConfig, PixelFormat, PixelWriter, Rgb};

mod graphics;
use graphics::{Vector2D, draw_rectangle};

mod pci;
use pci::{DEVICES, read_bar, read_vendor_id_from_device, scan_all_bus};

mod x86;

mod usb;
use usb::XhciController;

// TODO: should be replaced with safe rust code...
static mut CONSOLE: Option<Console> = None;
fn console() -> &'static mut Console {
    unsafe {
        #[allow(static_mut_refs)]
        CONSOLE.as_mut().unwrap()
    }
}
static mut PIXEL_WRITER: Option<BGRPixelWriter> = None;
fn pixel_writer() -> &'static mut BGRPixelWriter {
    unsafe {
        #[allow(static_mut_refs)]
        PIXEL_WRITER.as_mut().unwrap()
    }
}

pub type Result<T> = core::result::Result<T, &'static str>;

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
#[rustfmt::skip]
const MOUSE_CURSOR_SHAPE: [&str; MOUSE_CURSOR_HEIGHT] = [
   //0123456789abcde
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];

#[unsafe(no_mangle)]
extern "C" fn kernel_main(frame_buffer_config: &'static mut FrameBufferConfig) -> ! {
    main(frame_buffer_config).unwrap();
    loop {
        x86::halt();
    }
}

fn main(frame_buffer_config: &'static mut FrameBufferConfig) -> Result<()> {
    frame_buffer_config.frame_buffer().fill(0);
    let writer = match frame_buffer_config.pixel_format {
        PixelFormat::BGRR => BGRPixelWriter::new(frame_buffer_config),
        _ => unimplemented!(),
    };
    unsafe { PIXEL_WRITER = Some(writer) };

    let console = Console::new(Rgb::white(), Rgb::black());
    unsafe { CONSOLE = Some(console) };

    (0..MOUSE_CURSOR_HEIGHT).try_for_each(|dy| {
        (0..MOUSE_CURSOR_WIDTH).try_for_each(|dx| {
            if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).eq(&Some('@')) {
                pixel_writer().write(200 + dx as u32, 100 + dy as u32, Rgb::black())
            } else if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).eq(&Some('.')) {
                pixel_writer().write(200 + dx as u32, 100 + dy as u32, Rgb::white())
            } else {
                Ok(())
            }
        })
    })?;

    draw_rectangle(
        &Vector2D::new(100, 100),
        &Vector2D::new(100, 100),
        Rgb::red(),
    )?;

    scan_all_bus()?;

    let xhc_dev = unsafe { DEVICES }
        .iter()
        .filter_map(|dev| {
            let dev = (*dev)?;
            // println!("0x{:04x}", read_vendor_id_from_device(&dev));
            if dev.class_code.eq(&(0x0c, 0x03, 0x30))
            //&& read_vendor_id_from_device(&dev).eq(&0x8086)
            {
                return Some(dev);
            }
            None
        })
        .next()
        .ok_or("no xhci device")?;

    dbg!(&xhc_dev);

    let xhc_bar = read_bar(&xhc_dev, 0)?;
    let xhc_mmio_base = xhc_bar & !0xf;

    println!(
        "xhc_bar: 0x{:08x}, xhc_mmio_base: 0x{:08x}",
        xhc_bar, xhc_mmio_base
    );

    let mut xhc = XhciController::new(xhc_mmio_base as usize);
    xhc.initialize()?;
    xhc.run()?;
    xhc.configure_port();
    dbg!();
    usb::register_mouse_observer(mouse_observer);
    loop {
        xhc.process_event();
    }

    Ok(())
}

extern "C" fn mouse_observer(dx: i8, dy: i8) {
    dbg!(dx, dy);
}

#[panic_handler]
fn panic_impl(info: &core::panic::PanicInfo) -> ! {
    println!();
    println!("PANIC!!!");
    println!("{info}");

    loop {
        x86::halt();
    }
}
