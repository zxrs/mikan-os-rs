#![feature(str_from_raw_parts)]
#![no_std]
#![no_main]

use core::arch::asm;
use core::fmt::Write;

static mut CONSOLE: Option<Console<BGRPixelWriter>> = None;

#[macro_use]
mod macros;

mod fonts;

mod console;
use console::Console;

mod frame_buffer;
use frame_buffer::{BGRPixelWriter, FrameBufferConfig, PixelFormat, Rgb};

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
    frame_buffer_config.frame_buffer().fill(0);
    let writer = match frame_buffer_config.pixel_format {
        PixelFormat::BGRR => BGRPixelWriter::new(frame_buffer_config),
        _ => unimplemented!(),
    };

    let console = Console::new(writer, Rgb::white(), Rgb::black());
    unsafe { CONSOLE = Some(console) };

    (0..30).for_each(|i| {
        dbg!(i);
    });

    //(0..30).for_each(|i| {
    //    writeln!(&mut console, "line: {}", i).unwrap();
    //});

    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic_impl(info: &core::panic::PanicInfo) -> ! {
    println!();
    println!("PANIC!!!");
    println!("{info}");

    loop {
        unsafe { asm!("hlt") };
    }
}
