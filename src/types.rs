use arrayvec::ArrayVec;

/// Maximum number of frame rate ranges per format descriptor.
const MAX_FRAME_RATE_RANGES: usize = 8;

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

/// A rational number (numerator / denominator).
///
/// Used to represent frame rates (e.g. 30000/1000 = 30 fps) and
/// frame durations (e.g. 1000/30000 ≈ 0.033 s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ratio {
    pub numerator: u32,
    pub denominator: u32,
}

impl Ratio {
    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

/// Range of supported frame rates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameRateRange {
    pub min: Ratio,
    pub max: Ratio,
}

/// Describes a supported camera format.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatDescriptor {
    pub pixel_format: PixelFormat,
    pub size: Size,
    frame_rate_ranges: ArrayVec<FrameRateRange, MAX_FRAME_RATE_RANGES>,
}

impl FormatDescriptor {
    /// Create descriptors for a given format, automatically splitting across
    /// multiple [`FormatDescriptor`] values if the number of frame rate
    /// ranges exceeds the inline capacity.
    ///
    /// Most formats have only a handful of frame rate ranges, so this
    /// typically yields a single descriptor.
    pub(crate) fn from_ranges(
        pixel_format: PixelFormat,
        size: Size,
        frame_rate_ranges: impl IntoIterator<Item = FrameRateRange>,
    ) -> impl Iterator<Item = Self> {
        let mut iter = frame_rate_ranges.into_iter();
        core::iter::from_fn(move || {
            let mut chunk = ArrayVec::new();
            for range in iter.by_ref() {
                chunk.push(range);
                if chunk.is_full() {
                    break;
                }
            }
            if chunk.is_empty() {
                None
            } else {
                Some(FormatDescriptor {
                    pixel_format,
                    size,
                    frame_rate_ranges: chunk,
                })
            }
        })
    }

    /// The frame rate ranges supported by this format.
    pub fn frame_rate_ranges(&self) -> &[FrameRateRange] {
        &self.frame_rate_ranges
    }
}

/// Configuration for opening a camera stream.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub pixel_format: PixelFormat,
    pub size: Size,
    pub frame_rate: Ratio,
}
