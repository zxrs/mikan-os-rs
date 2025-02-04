use core::char::{REPLACEMENT_CHARACTER, decode_utf16};
use core::fmt;

/// [2.3.1 Data Types](https://uefi.org/specs/UEFI/2.11/02_Overview.html#data-types)
pub type EFIHandle = *const u8;

/// [2.3.1 Data Types](https://uefi.org/specs/UEFI/2.11/02_Overview.html#data-types)
pub struct EFIStatus(pub usize);

impl EFIStatus {
    pub fn is_success(&self) -> bool {
        self.0.eq(&0)
    }
}

/// [4.2.1 EFI_TABLE_HEADER](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#id4)
#[repr(C)]
pub struct EFITableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    reserved: u32,
}

pub struct CChar(*const u16);

impl CChar {
    pub fn from_ptr(ptr: *const u16) -> Self {
        Self(ptr)
    }

    pub fn len(&self) -> usize {
        let mut offset = 0;
        while unsafe { (*self.0.offset(offset as _)).ne(&0) } {
            offset += 1;
        }
        offset
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.0, self.len()) }
    }

    pub fn chars(&self) -> impl Iterator<Item = char> {
        decode_utf16(self.as_slice().iter().cloned()).map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
    }
}

impl fmt::Display for CChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.chars().try_for_each(|c| -> fmt::Result {
            write!(f, "{}", c)?;
            Ok(())
        })?;
        Ok(())
    }
}

/// [4.3.1 EFI_SYSTEM_TABLE](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#id6)
#[repr(C)]
pub struct EFISystemTable<'a> {
    pub header: EFITableHeader,
    pub firmware_vendor: CChar,
    pub firmware_revision: u32,
    pub console_in_handle: EFIHandle,
    pub con_in: *const u8,
    pub console_out_handle: EFIHandle,
    pub con_out: &'a EFISimpleTextOutputProtocol,
}

const _: () = {
    use core::mem::offset_of;
    ["size"][size_of::<EFISystemTable>() - 72];
    ["header"][offset_of!(EFISystemTable, header) - 0];
    ["vender"][offset_of!(EFISystemTable, firmware_vendor) - 24];
    ["revision"][offset_of!(EFISystemTable, firmware_revision) - 32];
    ["con_in_handle"][offset_of!(EFISystemTable, console_in_handle) - 40];
    ["conin"][offset_of!(EFISystemTable, con_in) - 48];
    ["con_out_handle"][offset_of!(EFISystemTable, console_out_handle) - 56];
    ["conout"][offset_of!(EFISystemTable, con_out) - 64];
};

/// [12.4.1 EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL](https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html#efi-simple-text-output-protocol)
#[repr(C)]
pub struct EFISimpleTextOutputProtocol {
    reset: EFIHandle,
    output_string: fn(&EFISimpleTextOutputProtocol, *const u16) -> EFIStatus,
    test_string: EFIHandle,
    query_mode: EFIHandle,
    set_mode: EFIHandle,
    set_attribute: EFIHandle,
    clear_screen: fn(&EFISimpleTextOutputProtocol) -> EFIStatus,
}

impl EFISimpleTextOutputProtocol {
    pub fn output_string(&self, c: *const u16) {
        (self.output_string)(self, c);
    }

    pub fn clear_screen(&self) {
        (self.clear_screen)(self);
    }
}

pub struct EFISimpleTextOutputProtocolWriter<'a>(&'a EFISimpleTextOutputProtocol);

impl<'a> EFISimpleTextOutputProtocolWriter<'a> {
    pub fn new(protocol: &'a EFISimpleTextOutputProtocol) -> Self {
        Self(protocol)
    }

    pub fn write_char(&mut self, c: u8) -> fmt::Result {
        let buf = [c as u16, 0];
        self.0.output_string(buf.as_ptr());
        Ok(())
    }
}

impl<'a> fmt::Write for EFISimpleTextOutputProtocolWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.bytes().try_for_each(|b| -> fmt::Result {
            if b.eq(&b'\n') {
                self.write_char(b'\r')?;
            }
            self.write_char(b)?;
            Ok(())
        })?;
        Ok(())
    }
}
