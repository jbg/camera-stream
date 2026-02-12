#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod frame;
pub mod types;

#[cfg(feature = "std")]
pub mod device;
#[cfg(feature = "std")]
pub mod error;
#[cfg(feature = "std")]
pub mod platform;
#[cfg(feature = "std")]
pub mod stream;

// Re-exports
#[doc(inline)]
pub use frame::*;
#[doc(inline)]
pub use types::*;

#[doc(inline)]
#[cfg(feature = "std")]
pub use device::*;
#[doc(inline)]
#[cfg(feature = "std")]
pub use error::*;
#[doc(inline)]
#[cfg(feature = "std")]
pub use stream::*;
