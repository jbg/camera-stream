use std::string::String;

use objc2::rc::Retained;
use objc2_av_foundation::{AVCaptureDevice, AVCaptureDeviceFormat, AVMediaTypeVideo};
use objc2_core_media::CMVideoFormatDescriptionGetDimensions;

use crate::device::{CameraDevice, CameraManager};
use crate::error::{Error, PlatformError};
use crate::platform::macos::stream::MacosCameraStream;
use crate::types::*;

/// macOS camera manager using AVFoundation.
#[derive(Default)]
pub struct MacosCameraManager;

impl CameraManager for MacosCameraManager {
    type Device = MacosCameraDevice;
    type Error = Error;

    fn discover_devices(&self) -> Result<Vec<Self::Device>, Self::Error> {
        let media_type = unsafe { AVMediaTypeVideo }.ok_or_else(|| {
            Error::Platform(PlatformError::Message(
                "AVMediaTypeVideo not available".into(),
            ))
        })?;

        #[allow(deprecated)]
        let devices = unsafe { AVCaptureDevice::devicesWithMediaType(media_type) };

        Ok(devices
            .iter()
            .map(|d| MacosCameraDevice::new(d.clone()))
            .collect())
    }

    fn default_device(&self) -> Result<Option<Self::Device>, Self::Error> {
        let media_type = unsafe { AVMediaTypeVideo }.ok_or_else(|| {
            Error::Platform(PlatformError::Message(
                "AVMediaTypeVideo not available".into(),
            ))
        })?;

        let device = unsafe { AVCaptureDevice::defaultDeviceWithMediaType(media_type) };
        Ok(device.map(MacosCameraDevice::new))
    }
}

/// Wraps an `AVCaptureDevice`.
pub struct MacosCameraDevice {
    pub(crate) device: Retained<AVCaptureDevice>,
    id_cache: String,
    name_cache: String,
}

impl MacosCameraDevice {
    pub(crate) fn new(device: Retained<AVCaptureDevice>) -> Self {
        let id_cache = unsafe { device.uniqueID() }.to_string();
        let name_cache = unsafe { device.localizedName() }.to_string();
        MacosCameraDevice {
            device,
            id_cache,
            name_cache,
        }
    }

    /// Access the underlying `AVCaptureDevice`.
    pub fn av_device(&self) -> &AVCaptureDevice {
        &self.device
    }
}

pub(crate) fn format_to_descriptor(format: &AVCaptureDeviceFormat) -> Option<FormatDescriptor> {
    let desc = unsafe { format.formatDescription() };
    let media_sub_type = unsafe { desc.media_sub_type() };
    let pixel_format = fourcc_to_pixel_format(media_sub_type)?;

    let dims = unsafe { CMVideoFormatDescriptionGetDimensions(&desc) };
    let size = Size {
        width: dims.width as u32,
        height: dims.height as u32,
    };

    let ranges = unsafe { format.videoSupportedFrameRateRanges() };
    let frame_rate_ranges: Vec<FrameRateRange> = ranges
        .iter()
        .map(|r| {
            let min_rate = unsafe { r.minFrameRate() };
            let max_rate = unsafe { r.maxFrameRate() };
            FrameRateRange {
                min: f64_to_frame_rate(min_rate),
                max: f64_to_frame_rate(max_rate),
            }
        })
        .collect();

    Some(FormatDescriptor {
        pixel_format,
        size,
        frame_rate_ranges,
    })
}

pub(crate) fn fourcc_to_pixel_format(fourcc: u32) -> Option<PixelFormat> {
    // kCVPixelFormatType values
    #[allow(clippy::mistyped_literal_suffixes)]
    match fourcc {
        0x34_32_30_76 => Some(PixelFormat::Nv12),   // '420v'
        0x34_32_30_66 => Some(PixelFormat::Nv12),   // '420f'
        0x79_75_76_32 => Some(PixelFormat::Yuyv),   // 'yuvs' / 'yuv2'
        0x32_76_75_79 => Some(PixelFormat::Uyvy),   // '2vuy'
        0x42_47_52_41 => Some(PixelFormat::Bgra32), // 'BGRA'
        0x6A_70_65_67 => Some(PixelFormat::Jpeg),   // 'jpeg'
        _ => None,
    }
}

pub(crate) fn pixel_format_to_fourcc(pf: &PixelFormat) -> u32 {
    #[allow(clippy::mistyped_literal_suffixes)]
    match pf {
        PixelFormat::Nv12 => 0x34_32_30_76,   // '420v'
        PixelFormat::Yuyv => 0x79_75_76_32,   // 'yuvs'
        PixelFormat::Uyvy => 0x32_76_75_79,   // '2vuy'
        PixelFormat::Bgra32 => 0x42_47_52_41, // 'BGRA'
        PixelFormat::Jpeg => 0x6A_70_65_67,   // 'jpeg'
    }
}

fn f64_to_frame_rate(fps: f64) -> FrameRate {
    // Express as integer ratio: fps ≈ numerator/1
    // For common rates, use 1000-based denominator for precision.
    let denominator = 1000u32;
    let numerator = (fps * denominator as f64).round() as u32;
    FrameRate {
        numerator,
        denominator,
    }
}

impl CameraDevice for MacosCameraDevice {
    type Stream = MacosCameraStream;
    type Error = Error;

    fn id(&self) -> &str {
        &self.id_cache
    }

    fn name(&self) -> &str {
        &self.name_cache
    }

    fn supported_formats(&self) -> Result<Vec<FormatDescriptor>, Self::Error> {
        let formats = unsafe { self.device.formats() };
        Ok(formats
            .iter()
            .filter_map(|f| format_to_descriptor(&f))
            .collect())
    }

    fn open(self, config: &StreamConfig) -> Result<Self::Stream, Self::Error> {
        MacosCameraStream::new(self.device, config)
    }
}
