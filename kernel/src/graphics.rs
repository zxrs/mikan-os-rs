use crate::{
    Result,
    frame_buffer::{PixelWriter, Rgb},
    pixel_writer,
};
use core::ops::Add;

#[derive(Debug, Copy, Clone)]
pub struct Vector2D<T: Copy + Clone> {
    pub x: T,
    pub y: T,
}

impl<T> Add<Vector2D<T>> for Vector2D<T>
where
    T: Add<Output = T> + Copy + Clone,
{
    type Output = Vector2D<T::Output>;

    fn add(self, rhs: Vector2D<T>) -> Vector2D<T::Output> {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
