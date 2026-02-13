use core::fmt;

#[cfg(target_os = "macos")]
use objc2::exception::Exception;
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_foundation::NSError;

/// Platform-specific error details.
///
/// On platforms that provide native error objects (e.g. `NSError` on macOS),
/// the original object is preserved. Use [`Display`](fmt::Display) (or
/// [`ToString::to_string`] when `alloc` is available) to obtain a
/// human-readable description.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformError {
    Message(&'static str),
    #[cfg(target_os = "macos")]
    NsError(Retained<NSError>),
    #[cfg(target_os = "macos")]
    ObjCException(Option<Retained<Exception>>),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => f.write_str(msg),
            #[cfg(target_os = "macos")]
            Self::NsError(e) => write!(f, "{e}"),
            #[cfg(target_os = "macos")]
            Self::ObjCException(Some(e)) => write!(f, "{e:?}"),
            #[cfg(target_os = "macos")]
            Self::ObjCException(None) => f.write_str("unknown Objective-C exception"),
        }
    }
}

impl core::error::Error for PlatformError {}

/// Top-level crate error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    DeviceNotFound,
    UnsupportedFormat,
    AlreadyStarted,
    NotStarted,
    Platform(PlatformError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceNotFound => f.write_str("no such device"),
            Self::UnsupportedFormat => f.write_str("unsupported format"),
            Self::AlreadyStarted => f.write_str("stream already started"),
            Self::NotStarted => f.write_str("stream not started"),
            Self::Platform(e) => write!(f, "platform error: {e}"),
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Platform(e) => Some(e),
            _ => None,
        }
    }
}

impl From<PlatformError> for Error {
    fn from(e: PlatformError) -> Self {
        Self::Platform(e)
    }
}
