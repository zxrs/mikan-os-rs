use crate::Result;

unsafe extern "C" {
    #[link_name = "\u{1}_ZN3usb4xhci10ControllerC1Em"]
    pub fn usb_xhci_Controller_Controller(this: *mut usb_xhci_Controller, mmio_base: usize);
}

unsafe extern "C" {
    fn UsbXhciController_initialize(c_impl: *mut usb_xhci_Controller) -> i32;
    fn UsbXhciController_run(c_impl: *mut usb_xhci_Controller) -> i32;
    fn UsbXhciController_configurePort(c_impl: *mut usb_xhci_Controller);
    fn UsbXhciController_ProcessXhcEvent(c_impl: *mut usb_xhci_Controller) -> i32;
    fn RegisterMouseObserver(cb: MouseObserver);
}

#[repr(C)]
#[derive(Debug)]
pub struct usb_xhci_Controller {
    pub mmio_base_: usize,
    pub cap_: *mut u8,
    pub op_: *mut u8,
    pub max_ports_: u8,
    pub devmgr_: usb_xhci_DeviceManager,
    pub cr_: usb_xhci_Ring,
    pub er_: usb_xhci_EventRing,
}

impl usb_xhci_Controller {
    pub fn new(mmio_base: usize) -> Self {
        let mut __bindgen_tmp = ::core::mem::MaybeUninit::uninit();
        unsafe { usb_xhci_Controller_Controller(__bindgen_tmp.as_mut_ptr(), mmio_base) };
        unsafe { __bindgen_tmp.assume_init() }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct usb_xhci_DeviceManager {
    pub device_context_pointers_: *mut *mut u8,
    pub max_slots_: usize,
    pub devices_: *mut *mut u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct usb_xhci_Ring {
    pub buf_: *mut u8,
    pub buf_size_: usize,
    pub cycle_bit_: bool,
    pub write_index_: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct usb_xhci_EventRing {
    pub buf_: *mut u8,
    pub buf_size_: usize,
    pub cycle_bit_: bool,
    pub erst_: *mut u8,
    pub interrupter_: *mut u8,
}

type MouseObserver = extern "C" fn(i8, i8);

pub struct XhciController {
    c_impl: usb_xhci_Controller,
}

unsafe impl Send for XhciController {}

impl XhciController {
    pub fn new(mmio_base: usize) -> Self {
        Self {
            c_impl: usb_xhci_Controller::new(mmio_base),
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        let error = unsafe { UsbXhciController_initialize(&mut self.c_impl) };
        if error > 0 {
            return Err("failed to initialize xhci usb controller.");
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let error = unsafe { UsbXhciController_run(&mut self.c_impl) };
        if error > 0 {
            return Err("failed to run xhci usb controller.");
        }
        Ok(())
    }

    pub fn configure_port(&mut self) {
        unsafe { UsbXhciController_configurePort(&mut self.c_impl) };
    }

    pub fn process_event(&mut self) -> i32 {
        unsafe { UsbXhciController_ProcessXhcEvent(&mut self.c_impl) }
    }
}

pub fn register_mouse_observer(cb: MouseObserver) {
    unsafe { RegisterMouseObserver(cb) };
}
