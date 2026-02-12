use crate::types::{PixelFormat, Size};

/// A single plane of image data.
pub struct Plane<'a> {
    pub data: &'a [u8],
    pub bytes_per_row: usize,
}

/// A presentation timestamp from the platform's media clock.
///
/// The interpretation of the underlying value depends on the platform.
/// Use [`as_secs_f64()`](Timestamp::as_secs_f64) for a portable
/// approximation, or access the platform-specific timestamp type
/// (via the [`Frame::Timestamp`] associated type) for full precision.
pub trait Timestamp {
    /// Seconds since an unspecified epoch (lossy convenience).
    fn as_secs_f64(&self) -> f64;
}

/// A borrowed video frame. Lifetime tied to callback scope (zero-copy).
pub trait Frame {
    type Timestamp: Timestamp;

    fn pixel_format(&self) -> PixelFormat;
    fn size(&self) -> Size;
    fn planes(&self) -> &[Plane<'_>];
    fn timestamp(&self) -> Self::Timestamp;
}
