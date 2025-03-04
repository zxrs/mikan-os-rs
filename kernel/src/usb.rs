use crate::Result;

unsafe extern "C" {
    #[link_name = "\u{1}_ZN3usb4xhci10ControllerC1Em"]
    fn usb_xhci_Controller_Controller(this: *mut UsbXhciController, mmio_base: usize);

    fn UsbXhciController_initialize(c_impl: *mut UsbXhciController) -> i32;
    fn UsbXhciController_run(c_impl: *mut UsbXhciController) -> i32;
    fn UsbXhciController_configurePort(c_impl: *mut UsbXhciController);
    fn UsbXhciController_ProcessXhcEvent(c_impl: *mut UsbXhciController) -> i32;
    fn UsbXhciController_PrimaryEventRing_HasFront(c_impl: *mut UsbXhciController) -> bool;
    fn RegisterMouseObserver(cb: MouseObserver);
}

pub static mut XHCI: Option<XhciController> = None;

pub fn init(mmio_base: usize) {
    let xhci = XhciController::new(mmio_base);
    unsafe { XHCI = Some(xhci) };
}

pub fn xhc() -> &'static mut XhciController {
    #[allow(static_mut_refs)]
    unsafe {
        XHCI.as_mut().unwrap()
    }
}

#[repr(C)]
#[derive(Debug)]
struct UsbXhciController {
    _padding: [u8; 128],
}

impl UsbXhciController {
    fn new(mmio_base: usize) -> Self {
        let mut __bindgen_tmp = ::core::mem::MaybeUninit::uninit();
        unsafe { usb_xhci_Controller_Controller(__bindgen_tmp.as_mut_ptr(), mmio_base) };
        unsafe { __bindgen_tmp.assume_init() }
    }
}

type MouseObserver = extern "C" fn(i8, i8);

pub struct XhciController {
    c_impl: UsbXhciController,
}

unsafe impl Send for XhciController {}

impl XhciController {
    pub fn new(mmio_base: usize) -> Self {
        Self {
            c_impl: UsbXhciController::new(mmio_base),
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

    pub fn process_event_ring_has_front(&mut self) -> bool {
        unsafe { UsbXhciController_PrimaryEventRing_HasFront(&mut self.c_impl) }
    }
}

pub fn register_mouse_observer(cb: MouseObserver) {
    unsafe { RegisterMouseObserver(cb) };
}
