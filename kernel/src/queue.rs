use crate::Result;

pub struct ArrayQueue<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> ArrayQueue<T, N> {
    pub fn new() -> Self {
        todo!()
    }

    pub fn push(&self, value: T) -> Result<()> {
        todo!()
    }

    pub fn pop(&self) -> Result<T> {
        todo!()
    }

    pub fn count(&self) -> usize {
        todo!()
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }
}
