#![allow(unused)]

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

impl<T> Vector2D<T>
where
    T: Copy + Clone,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Add<Vector2D<T>> for Vector2D<T>
where
    T: Add<Output = T> + Copy + Clone,
{
    type Output = Vector2D<T>;

    fn add(self, rhs: Vector2D<T>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub fn fill_rectangle(pos: &Vector2D<u32>, size: &Vector2D<u32>, color: Rgb) -> Result<()> {
    (0..size.y).try_for_each(|dy| {
        (0..size.x).try_for_each(|dx| pixel_writer().write(pos.x + dx, pos.y + dy, color))
    })
}

pub fn draw_rectangle(pos: &Vector2D<u32>, size: &Vector2D<u32>, color: Rgb) -> Result<()> {
    (0..size.x).try_for_each(|dx| {
        pixel_writer().write(pos.x + dx, pos.y, color)?;
        pixel_writer().write(pos.x + dx, pos.y + size.y - 1, color)?;
        Ok(())
    })?;
    (0..size.y).try_for_each(|dy| {
        pixel_writer().write(pos.x, pos.y + dy, color)?;
        pixel_writer().write(pos.x + size.x - 1, pos.y + dy, color)?;
        Ok(())
    })?;
    Ok(())
}
