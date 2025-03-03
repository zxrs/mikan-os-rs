
char xhc_buf[sizeof(usb::xhci::Controller)];
usb::xhci::Controller* xhc;

extern "C" {
    typedef struct {
        usb::xhci::Controller* controller;
    } XhciController;

    XhciController UsbXhciController(const uint64_t xhc_mmio_base) {
        xhc = new(xhc_buf) usb::xhci::Controller(xhc_mmio_base);
        return {xhc};
    }
}
