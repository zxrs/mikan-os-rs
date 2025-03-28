use core::char::{REPLACEMENT_CHARACTER, decode_utf16};
use core::fmt;
use core::ptr;
pub use share::memory_map::*;

pub type Result<T> = core::result::Result<T, &'static str>;

/// [2.3.1 Data Types](https://uefi.org/specs/UEFI/2.11/02_Overview.html#data-types)
pub type EFIHandle = *const u8;

/// [2.3.1 Data Types](https://uefi.org/specs/UEFI/2.11/02_Overview.html#data-types)
pub struct EFIStatus(pub usize);

#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
pub struct GUID {
    data0: u32,
    data1: u16,
    data2: u16,
    data3: [u8; 8],
}

/// [12.9.2. EFI_GRAPHICS_OUTPUT_PROTOCOL](https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html#efi-graphics-output-protocol)
const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: GUID = GUID {
    data0: 0x9042a9de,
    data1: 0x23dc,
    data2: 0x4a38,
    data3: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

/// [9.1.1. EFI_LOADED_IMAGE_PROTOCOL](https://uefi.org/specs/UEFI/2.11/09_Protocols_EFI_Loaded_Image.html#id3)
const EFI_LOADED_IMAGE_PROTOCOL_GUID: GUID = GUID {
    data0: 0x5b1b31a1,
    data1: 0x9562,
    data2: 0x11d2,
    data3: [0x8e, 0x3f, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

/// [13.4.1. EFI_SIMPLE_FILE_SYSTEM_PROTOCOL](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-simple-file-system-protocol)
const EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: GUID = GUID {
    data0: 0x0964e5b22,
    data1: 0x6459,
    data2: 0x11d2,
    data3: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

/// [13.5.16. EFI_FILE_INFO](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-file-info)
const EFI_FILE_INFO_GUID: GUID = GUID {
    data0: 0x09576e92,
    data1: 0x6d3f,
    data2: 0x11d2,
    data3: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

impl EFIStatus {
    pub fn is_success(&self) -> bool {
        self.0.eq(&0)
    }

    pub fn ok(&self) -> Result<()> {
        if self.is_success() {
            Ok(())
        } else {
            Err("failed")
        }
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
    #[allow(dead_code)]
    pub fn from_ptr(ptr: *const u16) -> Self {
        Self(ptr)
    }

    pub fn len(&self) -> usize {
        let mut offset = 0;
        while unsafe { (*self.0.add(offset)).ne(&0) } {
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
    con_in: EFIHandle,
    console_out_handle: EFIHandle,
    pub con_out: &'a EFISimpleTextOutputProtocol,
    standard_error_handle: EFIHandle,
    std_err: EFIHandle,
    runtime_services: EFIHandle,
    pub boot_services: &'a EFIBootServices,
}

const _: () = {
    use core::mem::offset_of;
    ["size"][size_of::<EFISystemTable>() - 104];
    ["header"][offset_of!(EFISystemTable, header)];
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

impl fmt::Write for EFISimpleTextOutputProtocolWriter<'_> {
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

/// [7.1.8. EFI_BOOT_SERVICES.RaiseTPL()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-raisetpl)
#[derive(Debug)]
#[repr(transparent)]
pub struct EFITpl(usize);

#[allow(dead_code)]
#[allow(non_snake_case)]
impl EFITpl {
    const APPLICATION: usize = 4;
    const CALLBACK: usize = 8;
    const NOTIFY: usize = 16;
    const HIGH_LEVEL: usize = 31;

    pub const fn Application() -> Self {
        Self(Self::APPLICATION)
    }

    pub const fn Callback() -> Self {
        Self(Self::CALLBACK)
    }

    pub const fn Notify() -> Self {
        Self(Self::NOTIFY)
    }

    pub const fn HighLevel() -> Self {
        Self(Self::HIGH_LEVEL)
    }
}

pub type EFIEvent = *mut u8;

#[allow(dead_code)]
#[repr(i32)]
pub enum EFITimerDelay {
    Cancel = 0,
    Periodic = 1,
    Relative = 2,
}

#[repr(transparent)]
pub struct EFIEventType(u32);

#[allow(non_snake_case, unused)]
impl EFIEventType {
    const TIMER: u32 = 0x80000000;

    pub const fn Timer() -> Self {
        Self(Self::TIMER)
    }
}

#[allow(unused)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum EFIAllocateType {
    AllocateAnyPages = 0,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType,
}

pub type EFIPhysicalAddress = u64;

/// [4.4.1. EFI_BOOT_SERVICES](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#efi-boot-services)
#[repr(C)]
pub struct EFIBootServices {
    header: EFITableHeader,
    #[doc(hidden)]
    _padding0: [EFIHandle; 2],
    /// [7.2.1. EFI_BOOT_SERVICES.AllocatePages()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-allocatepages)
    allocate_pages: fn(EFIAllocateType, EFIMemoryType, usize, *mut EFIPhysicalAddress) -> EFIStatus,
    free_pages: EFIHandle,
    /// [7.2.3. EFI_BOOT_SERVICES.GetMemoryMap()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-getmemorymap)
    get_memory_map: fn(&mut usize, *mut u8, &mut usize, &mut usize, &mut u32) -> EFIStatus,
    /// [7.2.4. EFI_BOOT_SERVICES.AllocatePool()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-allocatepool)
    allocate_pool: fn(EFIMemoryType, usize, *mut *mut u8) -> EFIStatus,
    #[doc(hidden)]
    _padding1: EFIHandle,
    create_event: fn(EFIEventType, EFITpl, *const u8, *const u8, &mut EFIEvent) -> EFIStatus,
    set_timer: fn(EFIEvent, EFITimerDelay, u64) -> EFIStatus,
    wait_for_event: fn(usize, &[EFIEvent], &mut usize) -> EFIStatus,
    #[doc(hidden)]
    _padding2: [EFIHandle; 16],
    /// [7.4.6. EFI_BOOT_SERVICES.ExitBootServices()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-exitbootservices)
    exit_boot_services: fn(EFIHandle, usize) -> EFIStatus,
    #[doc(hidden)]
    _padding3: [EFIHandle; 5],
    /// [7.3.9. EFI_BOOT_SERVICES.OpenProtocol()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-openprotocol)
    open_protocol: fn(EFIHandle, &GUID, *mut *mut u8, EFIHandle, EFIHandle, u32) -> EFIStatus,
    #[doc(hidden)]
    _padding4: [EFIHandle; 4],
    /// [7.3.16. EFI_BOOT_SERVICES.LocateProtocol()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-locateprotocol)
    locate_protocol: fn(&GUID, *const u8, *mut *mut u8) -> EFIStatus,
    #[doc(hidden)]
    _padding5: [EFIHandle; 3],
    /// [7.5.3. EFI_BOOT_SERVICES.CopyMem()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-copymem)
    pub copy_mem: fn(*mut u8, *const u8, usize) -> EFIStatus,
    /// [7.5.4. EFI_BOOT_SERVICES.SetMem()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-setmem)
    pub set_mem: fn(*const u8, usize, u8) -> EFIStatus,
}

const _: () = {
    use core::mem::offset_of;
    ["exit_boot_services"][offset_of!(EFIBootServices, exit_boot_services) - 232];
    ["open_protocol"][offset_of!(EFIBootServices, open_protocol) - 280];
    ["locate_protocol"][offset_of!(EFIBootServices, locate_protocol) - 320];
};

impl EFIBootServices {
    const EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: u32 = 0x00000001;

    pub fn allocate_pages(
        &self,
        typ: EFIAllocateType,
        memory_type: EFIMemoryType,
        pages: usize,
        address: EFIPhysicalAddress,
    ) -> Result<EFIPhysicalAddress> {
        let mut address = address;
        (self.allocate_pages)(typ, memory_type, pages, &mut address).ok()?;
        Ok(address)
    }

    pub fn get_memory_map(&self) -> Result<MemoryMap> {
        let mut memory_map = MemoryMap::default();
        (self.get_memory_map)(
            &mut memory_map.size,
            memory_map.buf.as_mut_ptr(),
            &mut memory_map.map_key,
            &mut memory_map.descriptor_size,
            &mut memory_map.version,
        )
        .ok()?;
        Ok(memory_map)
    }

    pub fn allocate_pool(&self, typ: EFIMemoryType, size: usize) -> Result<&[u8]> {
        let mut p = ptr::null_mut();
        (self.allocate_pool)(typ, size, &mut p).ok()?;
        let slice = unsafe { core::slice::from_raw_parts(p, size) };
        Ok(slice)
    }

    #[allow(unused)]
    pub fn create_event(&self, typ: EFIEventType, tpl: EFITpl) -> Result<EFIEvent> {
        let mut event = ptr::null_mut();
        (self.create_event)(typ, tpl, ptr::null(), ptr::null(), &mut event).ok()?;
        Ok(event)
    }

    #[allow(unused)]
    pub fn set_timer(&self, event: EFIEvent, typ: EFITimerDelay, delay: u64) -> Result<()> {
        (self.set_timer)(event, typ, delay).ok()
    }

    #[allow(unused)]
    pub fn wait_for_event(&self, events: &[EFIEvent]) -> Result<()> {
        let mut idx = 0;
        (self.wait_for_event)(events.len(), events, &mut idx).ok()
    }

    pub fn exit_boot_services(&self, image_handle: EFIHandle, map_key: usize) -> Result<()> {
        (self.exit_boot_services)(image_handle, map_key).ok()
    }

    pub fn locate_protocol<'a, T: Guid>(&self) -> Result<&'a T> {
        let mut p = ptr::null_mut();
        (self.locate_protocol)(&T::guid(), ptr::null(), &mut p).ok()?;
        Ok(unsafe { &*(p as *mut T) })
    }

    pub fn open_protocol<'a, T: Guid>(
        &self,
        handle: EFIHandle,
        agent_handle: EFIHandle,
    ) -> Result<&'a T> {
        let mut p = ptr::null_mut();
        (self.open_protocol)(
            handle,
            &T::guid(),
            &mut p,
            agent_handle,
            ptr::null(),
            Self::EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
        )
        .ok()?;
        Ok(unsafe { &*(p as *mut T) })
    }

    // pub fn copy_mem(&self, dst: &mut [u8], src: &[u8]) -> Result<()> {
    //     (self.copy_mem)(dst.as_mut_ptr(), src.as_ptr(), src.len()).ok()
    // }

    // pub fn set_mem(&self, buffer: &mut [u8], value: u8) -> Result<()> {
    //     (self.set_mem)(buffer.as_mut_ptr(), buffer.len(), value).ok()
    // }
}

/// [12.9.2. EFI_GRAPHICS_OUTPUT_PROTOCOL](https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html#efi-graphics-output-protocol)
#[repr(C)]
pub struct EFIGraphicsOutputProtocol<'a> {
    #[doc(hidden)]
    _padding: [EFIHandle; 3],
    pub mode: &'a EFIGraphicsOutputProtocolMode<'a>,
}

pub trait Guid {
    fn guid() -> GUID;
}

impl Guid for EFIGraphicsOutputProtocol<'_> {
    fn guid() -> GUID {
        EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct EFIGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    pub info: &'a EFIGraphicsOutputProtocolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize,
    pub frame_buffer_size: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct EFIGraphicsOutputProtocolPixelInfo {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub reserved_mask: u32,
    pub pixels_per_scan_line: u32,
}

#[repr(transparent)]
pub struct FileMode(u64);

#[allow(non_snake_case, unused)]
impl FileMode {
    const READ: u64 = 0x1;
    const WRITE: u64 = 0x2;
    const CREATE: u64 = 0x8000_0000_0000_0000;

    pub fn Read() -> Self {
        Self(Self::READ)
    }

    pub fn Write() -> Self {
        Self(Self::WRITE)
    }

    pub fn Create() -> Self {
        Self(Self::CREATE)
    }
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct FileAttributes(u64);

#[allow(non_snake_case, unused)]
impl FileAttributes {
    const READ_ONLY: u64 = 0x1;
    const HIDDEN: u64 = 0x2;
    const SYSTEM: u64 = 0x4;
    const RESERVED: u64 = 0x8;
    const DIRECTORY: u64 = 0x10;
    const ARCHIVE: u64 = 0x20;
    const VALID_ATTR: u64 = 0x37;

    pub fn ReadOnly() -> Self {
        Self(Self::READ_ONLY)
    }

    pub fn Hidden() -> Self {
        Self(Self::HIDDEN)
    }

    pub fn System() -> Self {
        Self(Self::SYSTEM)
    }

    pub fn Reserved() -> Self {
        Self(Self::RESERVED)
    }

    pub fn Directory() -> Self {
        Self(Self::DIRECTORY)
    }

    pub fn Archive() -> Self {
        Self(Self::ARCHIVE)
    }

    pub fn ValidAttr() -> Self {
        Self(Self::VALID_ATTR)
    }
}

/// [13.5.1. EFI_FILE_PROTOCOL](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-file-protocol)
#[repr(C)]
#[derive(Debug)]
pub struct EFIFileProtocol {
    revision: u64,
    /// [13.5.2. EFI_FILE_PROTOCOL.Open()](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-file-protocol-open)
    open: fn(*const EFIFileProtocol, *mut *mut EFIFileProtocol, CChar, u64, u64) -> EFIStatus,
    close: EFIHandle,
    delete: EFIHandle,
    /// [13.5.5. EFI_FILE_PROTOCOL.Read()](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#id28)
    read: fn(*const EFIFileProtocol, *mut usize, *mut usize) -> EFIStatus,
    #[doc(hidden)]
    _padding0: [EFIHandle; 3],
    /// [13.5.13. EFI_FILE_PROTOCOL.GetInfo()](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-file-protocol-getinfo)
    get_info: fn(*const EFIFileProtocol, &GUID, *mut usize, *mut u8) -> EFIStatus,
}

const _: () = {
    use core::mem::offset_of;
    ["get_info"][offset_of!(EFIFileProtocol, get_info) - 64];
};

impl EFIFileProtocol {
    pub fn open(
        &self,
        file_name: CChar,
        mode: FileMode,
        attributes: FileAttributes,
    ) -> Result<&EFIFileProtocol> {
        let mut p = ptr::null_mut();
        (self.open)(self, &mut p, file_name, mode.0, attributes.0).ok()?;
        Ok(unsafe { &*p })
    }

    pub fn read(&self, size: usize, address: usize) -> Result<&[u8]> {
        let mut buffer_size = size;
        (self.read)(self, &mut buffer_size, address as *mut _).ok()?;
        Ok(unsafe { core::slice::from_raw_parts(address as *const u8, buffer_size) })
    }

    pub fn get_info<T: Guid>(&self, file_info_buffer: &mut [u8]) -> Result<&T> {
        let mut len = file_info_buffer.len();
        (self.get_info)(self, &T::guid(), &mut len, file_info_buffer.as_mut_ptr()).ok()?;
        Ok(unsafe { &*(file_info_buffer.as_ptr() as *const T) })
    }
}

/// [9.1.1. EFI_LOADED_IMAGE_PROTOCOL](https://uefi.org/specs/UEFI/2.11/09_Protocols_EFI_Loaded_Image.html#id3)
#[repr(C)]
pub struct EFILoadedImageProtocol<'a> {
    revision: u32,
    parent_handle: EFIHandle,
    system_table: &'a EFISystemTable<'a>,
    device_handle: EFIHandle,
}

impl EFILoadedImageProtocol<'_> {
    pub fn device_handle(&self) -> EFIHandle {
        self.device_handle
    }
}

impl Guid for EFILoadedImageProtocol<'_> {
    fn guid() -> GUID {
        EFI_LOADED_IMAGE_PROTOCOL_GUID
    }
}

/// [13.4.1. EFI_SIMPLE_FILE_SYSTEM_PROTOCOL](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-simple-file-system-protocol)
#[repr(C)]
#[derive(Debug)]
pub struct EFISimpleFileSystemProtocol {
    revision: u64,
    open_volume: fn(*const EFISimpleFileSystemProtocol, *mut *mut EFIFileProtocol) -> EFIStatus,
}

impl EFISimpleFileSystemProtocol {
    pub fn open_volume(&self) -> Result<&EFIFileProtocol> {
        let mut p = ptr::null_mut();
        (self.open_volume)(self, &mut p).ok()?;
        Ok(unsafe { &*p })
    }
}

impl Guid for EFISimpleFileSystemProtocol {
    fn guid() -> GUID {
        EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID
    }
}

/// [13.5.16. EFI_FILE_INFO](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html#efi-file-info)
#[repr(C)]
pub struct EFIFileInfo {
    pub size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: EFITime,
    pub last_access_time: EFITime,
    pub modification_time: EFITime,
    pub attributes: u64,
    pub file_name: CChar,
}

impl Guid for EFIFileInfo {
    fn guid() -> GUID {
        EFI_FILE_INFO_GUID
    }
}

/// [8.3.1. GetTime()](https://uefi.org/specs/UEFI/2.11/08_Services_Runtime_Services.html#gettime)
#[repr(C)]
#[derive(Debug)]
pub struct EFITime {
    pub year: u16,  // 1900 - 9999
    pub month: u8,  // 1 - 12
    pub day: u8,    // 1 - 31
    pub hour: u8,   // 0 - 23
    pub minute: u8, // 0 - 59
    pub second: u8, // 0 - 59
    #[doc(hidden)]
    _pad0: u8,
    pub nano_sec: u32,  // 0 - 999_999_999
    pub time_zone: i32, // -1440 - 1440 or 2047
    pub day_light: u8,
    #[doc(hidden)]
    _pad1: u8,
}
