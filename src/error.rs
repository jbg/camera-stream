/// Platform-specific error details.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlatformError {
    #[error("{0}")]
    Message(String),
}

/// Top-level crate error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("no such device")]
    DeviceNotFound,
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("stream already started")]
    AlreadyStarted,
    #[error("stream not started")]
    NotStarted,
    #[error("platform error: {0}")]
    Platform(#[from] PlatformError),
}
