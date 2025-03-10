use crate::x86;

const PAGE_DIRECTORY_COUNT: usize = 64;
const PAGE_SIZE_4K: u64 = 4096;
const PAGE_SIZE_2M: u64 = 512 * PAGE_SIZE_4K;
const PAGE_SIZE_1G: u64 = 512 * PAGE_SIZE_2M;

#[repr(align(4096))]
struct Pml4Table([u64; 512]);

#[repr(align(4096))]
struct PdpTable([u64; 512]);

#[repr(align(4096))]
struct PageDirectory([[u64; 512]; PAGE_DIRECTORY_COUNT]);

static mut PML4_TABLE: Pml4Table = Pml4Table([0; 512]);
fn pml4_table() -> &'static mut Pml4Table {
    #[allow(static_mut_refs)]
    unsafe {
        &mut PML4_TABLE
    }
}

static mut PDP_TABLE: PdpTable = PdpTable([0; 512]);
fn pdp_table() -> &'static mut PdpTable {
    #[allow(static_mut_refs)]
    unsafe {
        &mut PDP_TABLE
    }
}

static mut PAGE_DIRECTORY: PageDirectory = PageDirectory([[0; 512]; PAGE_DIRECTORY_COUNT]);
fn page_directory() -> &'static mut PageDirectory {
    #[allow(static_mut_refs)]
    unsafe {
        &mut PAGE_DIRECTORY
    }
}

pub fn setup_identity_page_table() {
    pml4_table().0[0] = (pdp_table().0[0] as *mut PdpTable as u64) | 0x003;
    (0..page_directory().0.len()).for_each(|i_pdpt| {
        pdp_table().0[i_pdpt] = &page_directory().0[i_pdpt] as *const _ as u64 | 0x003;
        (0..512).for_each(|i_pd| {
            page_directory().0[i_pdpt][i_pd] =
                (i_pdpt as u64 * PAGE_SIZE_1G + i_pd as u64 * PAGE_SIZE_2M) | 0x083;
        });
    });

    x86::set_cr3(pml4_table() as *mut Pml4Table as u64);
}
