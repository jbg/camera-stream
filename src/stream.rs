use crate::frame::Frame;

/// Callback-based frame delivery.
pub trait CameraStream {
    type Frame<'a>: Frame
    where
        Self: 'a;
    type Error: std::error::Error;

    /// Start streaming. Callback is invoked on a platform thread for each frame.
    fn start<F>(&mut self, callback: F) -> Result<(), Self::Error>
    where
        F: FnMut(&Self::Frame<'_>) + Send + 'static;

    fn stop(&mut self) -> Result<(), Self::Error>;
}
