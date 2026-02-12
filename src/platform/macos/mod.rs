use crate::error::{Error, PlatformError};

pub mod device;
pub mod ext;
pub mod frame;
pub mod stream;

/// Catch Objective-C exceptions and convert them to our Error type.
fn catch_objc<R>(f: impl FnOnce() -> R + std::panic::UnwindSafe) -> Result<R, Error> {
    objc2::exception::catch(f)
        .map_err(|exception| Error::Platform(PlatformError::ObjCException(exception)))
}
