use crate::Result;

#[const_trait]
trait Kib {
    fn kib(self) -> usize;
}

impl const Kib for usize {
    fn kib(self) -> usize {
        self * 1024
    }
}

#[const_trait]
trait Mib {
    fn mib(self) -> usize;
}

impl const Mib for usize {
    fn mib(self) -> usize {
        self * 1024.kib()
    }
}

#[const_trait]
trait Gib {
    fn gib(self) -> usize;
}

impl const Gib for usize {
    fn gib(self) -> usize {
        self * 1024.mib()
    }
}

const BYTES_PER_FRAME: usize = 4.kib();
const FULL_FRAME: FrameID = FrameID::new(usize::MAX);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameID {
    id: usize,
}

impl FrameID {
    pub const fn new(id: usize) -> Self {
        Self { id }
    }

    pub const fn id(&self) -> usize {
        self.id
    }

    pub fn frame(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.id() * BYTES_PER_FRAME) as *mut u8,
                BYTES_PER_FRAME,
            )
        }
    }
}

type MapLineType = u64;

struct BitmapMemoryManager {}

impl BitmapMemoryManager {
    const MAX_PHYSICAL_MEMORY_BYTES: usize = 128.gib();
    const FRAME_COUNT: usize = Self::MAX_PHYSICAL_MEMORY_BYTES / BYTES_PER_FRAME;
    const BITS_PER_MAP_LINE: usize = 8 * size_of::<u64>();

    fn allocate(num_frames: usize) -> Result<FrameID> {
        todo!()
    }

    fn free(start_frame: FrameID, num_frames: usize) -> Result<()> {
        todo!()
    }

    fn mark_allocated(start_frame: FrameID, num_frames: usize) {
        todo!()
    }
}
