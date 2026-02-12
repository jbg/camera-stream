use std::ffi::c_void;

use objc2_core_video::{
    CVPixelBuffer, CVPixelBufferGetBaseAddress, CVPixelBufferGetBaseAddressOfPlane,
    CVPixelBufferGetBytesPerRow, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight,
    CVPixelBufferGetHeightOfPlane, CVPixelBufferGetPixelFormatType, CVPixelBufferGetPlaneCount,
    CVPixelBufferGetWidth,
};

use crate::frame::{Frame, Plane, Timestamp};
use crate::platform::macos::device::fourcc_to_pixel_format;
use crate::types::{PixelFormat, Size};

/// A presentation timestamp mirroring Core Media's `CMTime`.
///
/// Preserves the full precision and semantics of the underlying `CMTime`,
/// including flags and epoch. For a quick seconds value, use
/// [`as_secs_f64()`](Timestamp::as_secs_f64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacosTimestamp {
    /// The numerator of the time value (ticks).
    pub value: i64,
    /// Ticks per second.
    pub timescale: i32,
    /// CMTime flags (valid, has been rounded, positive/negative infinity, indefinite).
    pub flags: u32,
    /// Distinguishes separate timelines that may restart from zero.
    pub epoch: i64,
}

impl Timestamp for MacosTimestamp {
    fn as_secs_f64(&self) -> f64 {
        if self.timescale > 0 {
            self.value as f64 / self.timescale as f64
        } else {
            0.0
        }
    }
}

/// A video frame backed by a `CVPixelBuffer`.
/// Only valid within the callback scope.
pub struct MacosFrame<'a> {
    pixel_buffer: &'a CVPixelBuffer,
    planes: Vec<Plane<'a>>,
    pixel_format: PixelFormat,
    size: Size,
    timestamp: MacosTimestamp,
}

impl<'a> MacosFrame<'a> {
    /// Create a frame from a locked pixel buffer.
    /// SAFETY: The pixel buffer base address must be locked for the lifetime 'a.
    pub(crate) unsafe fn from_locked_pixel_buffer(
        pixel_buffer: &'a CVPixelBuffer,
        timestamp: MacosTimestamp,
    ) -> Self {
        let width = CVPixelBufferGetWidth(pixel_buffer);
        let height = CVPixelBufferGetHeight(pixel_buffer);
        let fourcc = CVPixelBufferGetPixelFormatType(pixel_buffer);
        let pixel_format = fourcc_to_pixel_format(fourcc).unwrap_or(PixelFormat::Nv12);
        let size = Size {
            width: width as u32,
            height: height as u32,
        };

        let plane_count = CVPixelBufferGetPlaneCount(pixel_buffer);
        let planes = if plane_count == 0 {
            // Non-planar: single plane
            let base = CVPixelBufferGetBaseAddress(pixel_buffer);
            let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel_buffer);
            if base.is_null() {
                vec![]
            } else {
                let len = bytes_per_row * height;
                let data = unsafe { std::slice::from_raw_parts(base as *const u8, len) };
                vec![Plane {
                    data,
                    bytes_per_row,
                }]
            }
        } else {
            (0..plane_count)
                .filter_map(|i| {
                    let base = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, i);
                    if base.is_null() {
                        return None;
                    }
                    let bytes_per_row = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, i);
                    let h = CVPixelBufferGetHeightOfPlane(pixel_buffer, i);
                    let len = bytes_per_row * h;
                    let data = unsafe { std::slice::from_raw_parts(base as *const u8, len) };
                    Some(Plane {
                        data,
                        bytes_per_row,
                    })
                })
                .collect()
        };

        MacosFrame {
            pixel_buffer,
            planes,
            pixel_format,
            size,
            timestamp,
        }
    }

    /// Raw pointer to the backing `CVPixelBuffer` (escape hatch).
    pub fn pixel_buffer_ptr(&self) -> *const c_void {
        self.pixel_buffer as *const CVPixelBuffer as *const c_void
    }
}

impl<'a> Frame for MacosFrame<'a> {
    type Timestamp = MacosTimestamp;

    fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    fn size(&self) -> Size {
        self.size
    }

    fn planes(&self) -> &[Plane<'_>] {
        &self.planes
    }

    fn timestamp(&self) -> MacosTimestamp {
        self.timestamp
    }
}
