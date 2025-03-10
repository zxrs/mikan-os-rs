#![feature(str_from_raw_parts)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::fmt::Write;

#[macro_use]
mod macros;

#[rustfmt::skip]
r#mod!(fonts, console, frame_buffer, graphics, mouse, pci, usb, interrupt, queue, segment, x86, x86_descriptor, paging);

use console::console;
use frame_buffer::{BGRPixelWriter, FrameBufferConfig, PixelFormat, Rgb, pixel_writer};
use graphics::{Vector2D, draw_rectangle};
use interrupt::{
    IDT, InterruptDescriptor, InterruptFrame, InterruptVector, make_idt_attr,
    notify_end_of_interrupt, set_idt_entry,
};
use mouse::mouse_cursor;
use paging::setup_identity_page_table;
use pci::{DEVICES, read_bar, scan_all_bus};
use queue::ArrayQueue;
use segment::setup_segments;
use share::memory_map::{self, MemoryDescriptorVisitor, MemoryMap, memory_map};
use usb::xhc;
use x86_descriptor::DescriptorType;

type Result<T> = core::result::Result<T, &'static str>;

#[repr(align(16))]
struct KernelMainStack([u8; 1024 * 1024]);

static mut KERNEL_MAIN_STACK: KernelMainStack = KernelMainStack([0; 1024 * 1024]);
fn kernel_main_stack() -> &'static KernelMainStack {
    #[allow(static_mut_refs)]
    unsafe {
        &mut KERNEL_MAIN_STACK
    }
}

static mut MAIN_QUEUE: ArrayQueue<Message, 32> = ArrayQueue::new();
fn main_queue() -> &'static mut ArrayQueue<Message, 32> {
    #[allow(static_mut_refs)]
    unsafe {
        &mut MAIN_QUEUE
    }
}

#[derive(Debug, Clone, Copy)]
struct Message(MessageType);

#[derive(Debug, Clone, Copy)]
enum MessageType {
    InterruptXhci,
}

#[unsafe(no_mangle)]
extern "C" fn kernel_main(
    frame_buffer_config: &'static mut FrameBufferConfig,
    memory_map_: &'static MemoryMap,
) -> ! {
    frame_buffer::init(frame_buffer_config);
    console::init(Rgb::white(), Rgb::black());
    mouse::init(Rgb::black(), 200, 100).unwrap();
    memory_map::init(memory_map_);

    x86::switch_rsp(
        kernel_main_stack() as *const KernelMainStack as usize + 1024 * 1024,
        kernel_main_new_stack,
    );

    loop {
        x86::halt();
    }
}

fn kernel_main_new_stack() -> ! {
    main().unwrap();
    loop {
        x86::halt();
    }
}

fn main() -> Result<()> {
    setup_segments();

    const KERNEL_CS: u16 = 1 << 3;
    const KERNEL_SS: u16 = 2 << 3;

    x86::set_ds_all(0);
    x86::set_cs_ss(KERNEL_CS, KERNEL_SS);

    setup_identity_page_table();

    // let visitor = MemoryDescriptorVisitor::new(memory_map());
    // visitor.for_each(|d| {
    //     dbg!(d.typ);
    // });

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
        interrupt_handler_xhci as usize as u64,
        cs,
    );

    #[allow(static_mut_refs)]
    let param = x86::IdtParam {
        limit: size_of::<[InterruptDescriptor; 256]>() as u16 - 1,
        base: unsafe { &IDT[0] as *const InterruptDescriptor as usize },
    };
    x86::load_idt(&param);

    let bsp_local_apic_id = (unsafe { *(0xfee00020 as *const u32) } >> 24) as u8;
    dbg!(bsp_local_apic_id);

    pci::configure_msi_fixed_destination(
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

    x86::sti();

    usb::register_mouse_observer(mouse_observer);
    xhc.configure_port();

    loop {
        x86::cli();
        if main_queue().count() == 0 {
            x86::sti();
            x86::halt();
            continue;
        }

        let msg = main_queue().front();
        _ = main_queue().pop();
        x86::sti();

        match msg.0 {
            MessageType::InterruptXhci => {
                while xhc.process_event_ring_has_front() {
                    let _e = xhc.process_event();
                    // dbg!(e);
                }
            }
            msg => {
                dbg!("unknown message type:", msg);
            }
        }
    }

    Ok(())
}

extern "C" fn mouse_observer(dx: i8, dy: i8) {
    _ = mouse_cursor().move_relative(Vector2D::new(dx as i32, dy as i32));
}

extern "x86-interrupt" fn interrupt_handler_xhci(_: InterruptFrame) {
    _ = main_queue().push(Message(MessageType::InterruptXhci));
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
