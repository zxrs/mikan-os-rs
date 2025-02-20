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
use graphics::draw_rectangle;

mod graphics;
use graphics::Vector2D;

mod pci;

mod x86;

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
    frame_buffer_config.frame_buffer().fill(0);
    let writer = match frame_buffer_config.pixel_format {
        PixelFormat::BGRR => BGRPixelWriter::new(frame_buffer_config),
        _ => unimplemented!(),
    };
    unsafe { PIXEL_WRITER = Some(writer) };

    let console = Console::new(Rgb::white(), Rgb::black());
    unsafe { CONSOLE = Some(console) };

    (0..30).for_each(|i| {
        dbg!(i);
    });

    (0..MOUSE_CURSOR_HEIGHT).for_each(|dy| {
        (0..MOUSE_CURSOR_WIDTH).for_each(|dx| {
            if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).eq(&Some('@')) {
                pixel_writer()
                    .write(200 + dx as u32, 100 + dy as u32, Rgb::black())
                    .unwrap();
            } else if MOUSE_CURSOR_SHAPE[dy].chars().nth(dx).eq(&Some('.')) {
                pixel_writer()
                    .write(200 + dx as u32, 100 + dy as u32, Rgb::white())
                    .unwrap();
            }
        });
    });

    draw_rectangle(
        &Vector2D::new(100, 100),
        &Vector2D::new(100, 100),
        Rgb::red(),
    )
    .unwrap();

    //(0..30).for_each(|i| {
    //    writeln!(&mut console, "line: {}", i).unwrap();
    //});

    loop {
        x86::halt();
    }
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
