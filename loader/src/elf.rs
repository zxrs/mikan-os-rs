use super::{EFISystemTable, uefi::Result};

const EI_NIDENT: usize = 16;

#[repr(C)]
#[derive(Debug)]
pub struct Elf64_Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

impl Elf64_Ehdr {
    pub fn entry_addr(&self) -> u64 {
        self.e_entry
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64_Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

pub fn calc_load_address_range(ehdr: &Elf64_Ehdr) -> (u64, u64) {
    let p_phdr = (ehdr as *const Elf64_Ehdr as u64 + ehdr.e_phoff) as *mut Elf64_Phdr;
    let s = unsafe { core::slice::from_raw_parts(p_phdr, ehdr.e_phnum as usize) };
    let mut first = u64::MAX;
    let mut last = 0;
    for phdr in s.iter() {
        if phdr.p_type != 1 {
            // PT_LOAD: 1
            continue;
        }

        first = first.min(phdr.p_vaddr - phdr.p_offset);
        last = last.max(phdr.p_vaddr - phdr.p_offset + phdr.p_memsz);
    }
    (first, last)
}

pub fn copy_load_segments(system_table: &EFISystemTable, ehdr: &Elf64_Ehdr) -> Result<()> {
    let p_phdr = (ehdr as *const Elf64_Ehdr as usize + ehdr.e_phoff as usize) as *mut Elf64_Phdr;
    let s = unsafe { core::slice::from_raw_parts(p_phdr, ehdr.e_phnum as usize) };
    for phdr in s.iter() {
        if phdr.p_type != 1 {
            // PT_LOAD: 1
            continue;
        }

        let segm_in_file = ehdr as *const Elf64_Ehdr as usize + phdr.p_offset as usize;
        (system_table.boot_services.copy_mem)(
            phdr.p_vaddr as *mut u8,
            segm_in_file as *const u8,
            phdr.p_filesz as usize,
        );

        let remain_bytes = phdr.p_memsz - phdr.p_filesz;
        (system_table.boot_services.set_mem)(
            (phdr.p_vaddr + phdr.p_filesz) as *const u8,
            remain_bytes as usize,
            0,
        );
    }
    Ok(())
}
