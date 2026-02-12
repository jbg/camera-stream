#![allow(unused)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Pixel formats encountered across platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PixelFormat {
    Nv12,
    Yuyv,
    Uyvy,
    Bgra32,
    Jpeg,
}

/// Pixel dimensions of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Frame rate as a rational number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
    pub numerator: u32,
    pub denominator: u32,
}

impl FrameRate {
    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

/// Range of supported frame rates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameRateRange {
    pub min: FrameRate,
    pub max: FrameRate,
}

/// Describes a supported camera format.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq)]
pub struct FormatDescriptor {
    pub pixel_format: PixelFormat,
    pub size: Size,
    pub frame_rate_ranges: Vec<FrameRateRange>,
}

/// Configuration for opening a camera stream.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub pixel_format: PixelFormat,
    pub size: Size,
    pub frame_rate: FrameRate,
}
