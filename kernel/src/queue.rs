use crate::Result;

#[derive(Debug)]
pub struct ArrayQueue<T: Clone + Copy, const N: usize> {
    data: [T; N],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl<T: Clone + Copy, const N: usize> ArrayQueue<T, N> {
    const CAPACITY: usize = N;

    pub const fn new() -> Self {
        Self {
            data: unsafe { core::mem::zeroed() },
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<()> {
        if self.count == Self::CAPACITY {
            return Err("queue is full.");
        }

        self.data[self.write_pos] = value;
        self.count += 1;
        self.write_pos += 1;
        if self.write_pos == Self::CAPACITY {
            self.write_pos = 0;
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Result<()> {
        if self.count == 0 {
            return Err("queue is empty.");
        }

        self.count -= 1;
        self.read_pos += 1;
        if self.read_pos == Self::CAPACITY {
            self.read_pos = 0;
        }
        Ok(())
    }

    pub const fn front(&self) -> &T {
        &self.data[self.read_pos]
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    pub const fn capasity(&self) -> usize {
        Self::CAPACITY
    }
}

impl<T: Clone + Copy, const N: usize> Default for ArrayQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
