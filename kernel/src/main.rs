#![feature(str_from_raw_parts)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::fmt::Write;

#[macro_use]
mod macros;

#[rustfmt::skip]
r#mod!(fonts, console, frame_buffer, graphics, mouse, pci, x86, usb, interrupt);

use console::console;
use frame_buffer::{BGRPixelWriter, FrameBufferConfig, PixelFormat, Rgb, pixel_writer};
use graphics::{Vector2D, draw_rectangle};
use interrupt::{
    DescriptorType, IDT, InterruptFrame, InterruptVector, make_idt_attr, notify_end_of_interrupt,
    set_idt_entry,
};
use mouse::mouse_cursor;
use pci::{DEVICES, read_bar, scan_all_bus};
use usb::xhc;

pub type Result<T> = core::result::Result<T, &'static str>;

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
    frame_buffer::init(writer);
    console::init(Rgb::white(), Rgb::black());
    mouse::init(Rgb::black(), 200, 100)?;

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

    let cs = x86::get_cs();
    let attr = make_idt_attr(DescriptorType::InterruptGate(), 0, true, 0);
    set_idt_entry(
        unsafe { &mut IDT[InterruptVector::Xhci().get()] },
        attr,
        interrupt_handler_xhci as u64,
        cs,
    );

    #[allow(static_mut_refs)]
    let param = x86::IdtParam {
        limit: size_of_val(unsafe { &IDT }) as u16 - 1,
        base: unsafe { IDT.as_ptr().addr() },
    };
    x86::load_idt(&param);

    let bsp_local_apic_id = (unsafe { *core::ptr::null_mut::<u32>().add(0xfee00020) } >> 24) as u8;
    dbg!(bsp_local_apic_id);

    pci::configure_msi_fixed_destication(
        &xhc_dev,
        bsp_local_apic_id,
        pci::MSITriggerMode::Level,
        pci::MSIDeliveryMode::Fixed(),
        InterruptVector::Xhci().get() as u8,
        0,
    )?;

    let xhc_bar = read_bar(&xhc_dev, 0)?;
    let xhc_mmio_base = xhc_bar & !0xf;

    println!(
        "xhc_bar: 0x{:08x}, xhc_mmio_base: 0x{:08x}",
        xhc_bar, xhc_mmio_base
    );

    usb::init(xhc_mmio_base as usize);
    let xhc = xhc();
    xhc.initialize()?;
    xhc.run()?;
    xhc.configure_port();
    usb::register_mouse_observer(mouse_observer);

    Ok(())
}

extern "C" fn mouse_observer(dx: i8, dy: i8) {
    _ = mouse_cursor().move_relative(Vector2D::new(dx as i32, dy as i32));
}

extern "x86-interrupt" fn interrupt_handler_xhci(frame: &InterruptFrame) {
    let xhc = xhc();
    while xhc.process_event_ring_has_front() {
        if xhc.process_event() > 0 {
            dbg!();
        }
    }
    notify_end_of_interrupt();
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
