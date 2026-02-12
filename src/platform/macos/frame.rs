use core::time::Duration;
use std::ffi::c_void;

use objc2_core_video::{
    CVPixelBuffer, CVPixelBufferGetBaseAddress, CVPixelBufferGetBaseAddressOfPlane,
    CVPixelBufferGetBytesPerRow, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight,
    CVPixelBufferGetHeightOfPlane, CVPixelBufferGetPixelFormatType, CVPixelBufferGetPlaneCount,
    CVPixelBufferGetWidth,
};

use crate::frame::{Frame, Plane};
use crate::platform::macos::device::fourcc_to_pixel_format;
use crate::types::{PixelFormat, Resolution};

/// A video frame backed by a `CVPixelBuffer`.
/// Only valid within the callback scope.
pub struct MacosFrame<'a> {
    pixel_buffer: &'a CVPixelBuffer,
    planes: Vec<Plane<'a>>,
    pixel_format: PixelFormat,
    resolution: Resolution,
    timestamp: Duration,
}

impl<'a> MacosFrame<'a> {
    /// Create a frame from a locked pixel buffer.
    /// SAFETY: The pixel buffer base address must be locked for the lifetime 'a.
    pub(crate) unsafe fn from_locked_pixel_buffer(
        pixel_buffer: &'a CVPixelBuffer,
        timestamp: Duration,
    ) -> Self {
        let width = CVPixelBufferGetWidth(pixel_buffer);
        let height = CVPixelBufferGetHeight(pixel_buffer);
        let fourcc = CVPixelBufferGetPixelFormatType(pixel_buffer);
        let pixel_format = fourcc_to_pixel_format(fourcc).unwrap_or(PixelFormat::Nv12);
        let resolution = Resolution {
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
            resolution,
            timestamp,
        }
    }

    /// Raw pointer to the backing `CVPixelBuffer` (escape hatch).
    pub fn pixel_buffer_ptr(&self) -> *const c_void {
        self.pixel_buffer as *const CVPixelBuffer as *const c_void
    }
}

impl<'a> Frame for MacosFrame<'a> {
    fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    fn resolution(&self) -> Resolution {
        self.resolution
    }

    fn planes(&self) -> &[Plane<'_>] {
        &self.planes
    }

    fn timestamp(&self) -> Duration {
        self.timestamp
    }
}
