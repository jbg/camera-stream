use crate::stream::CameraStream;
use crate::types::{FormatDescriptor, StreamConfig};

/// Discover and inspect camera devices.
pub trait CameraManager {
    type Device: CameraDevice;
    type Error: std::error::Error;

    fn discover_devices(&self) -> Result<Vec<Self::Device>, Self::Error>;
    fn default_device(&self) -> Result<Option<Self::Device>, Self::Error>;
}

/// A camera device that can be inspected and opened.
pub trait CameraDevice {
    type Stream: CameraStream;
    type Error: std::error::Error;

    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn supported_formats(&self) -> Result<Vec<FormatDescriptor>, Self::Error>;
    fn open(self, config: &StreamConfig) -> Result<Self::Stream, Self::Error>;
}
