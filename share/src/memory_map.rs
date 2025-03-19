const MEMORY_MAP_SIZE: usize = 4096 * 4;

pub static mut MEMORY_MAP: Option<MemoryMap> = None;

pub fn init(memory_map: &'static MemoryMap) {
    unsafe { MEMORY_MAP = Some(*memory_map) };
}

pub fn memory_map() -> &'static MemoryMap {
    #[allow(static_mut_refs)]
    unsafe {
        MEMORY_MAP.as_ref().unwrap()
    }
}

#[allow(unused)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EFIMemoryType {
    ReservedMemoryType = 0,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    ACPIReclaimMemory,
    ACPIMemoryNVS,
    MemoryMappedIO,
    MemoryMappedIOPortSpace,
    PalCode,
    PersistentMemory,
    UnacceptedMemoryType,
    MaxMemoryType,
}

impl From<EFIMemoryType> for u32 {
    fn from(value: EFIMemoryType) -> Self {
        value as _
    }
}

/// [7.2.3. EFI_BOOT_SERVICES.GetMemoryMap()](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html#efi-boot-services-getmemorymap)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EFIMemoryDescriptor {
    pub typ: EFIMemoryType,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub size: usize,
    pub buf: [u8; MEMORY_MAP_SIZE],
    pub map_key: usize,
    pub descriptor_size: usize,
    pub version: u32,
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            size: MEMORY_MAP_SIZE,
            buf: [0; MEMORY_MAP_SIZE],
            map_key: 0,
            descriptor_size: 0,
            version: 0,
        }
    }
}

#[derive(Debug)]
pub struct MemoryDescriptorVisitor<'a> {
    memory_map: &'a MemoryMap,
    offset: usize,
}

impl<'a> MemoryDescriptorVisitor<'a> {
    pub fn new(memory_map: &'a MemoryMap) -> Self {
        Self {
            memory_map,
            offset: 0,
        }
    }
}

impl Iterator for MemoryDescriptorVisitor<'_> {
    type Item = EFIMemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.memory_map.size - self.memory_map.descriptor_size {
            return None;
        }

        let descriptor = self
            .memory_map
            .buf
            .get(self.offset)
            .map(|p| unsafe { *(p as *const u8 as *const EFIMemoryDescriptor) });

        self.offset += self.memory_map.descriptor_size;
        descriptor
    }
}

pub fn is_available(memory_type: EFIMemoryType) -> bool {
    memory_type == EFIMemoryType::BootServicesCode
        || memory_type == EFIMemoryType::BootServicesData
        || memory_type == EFIMemoryType::ConventionalMemory
}

pub const UEFI_PAGE_SIZE: i32 = 4096;
