use crate::types::{PixelFormat, Resolution};
use core::time::Duration;

/// A single plane of image data.
pub struct Plane<'a> {
    pub data: &'a [u8],
    pub bytes_per_row: usize,
}

/// A borrowed video frame. Lifetime tied to callback scope (zero-copy).
pub trait Frame {
    fn pixel_format(&self) -> PixelFormat;
    fn resolution(&self) -> Resolution;
    fn planes(&self) -> &[Plane<'_>];
    fn timestamp(&self) -> Duration;
}
