#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod frame;
pub mod types;

pub mod device;
pub mod error;
#[cfg(feature = "std")]
pub mod platform;
pub mod stream;

// Re-exports
#[doc(inline)]
pub use frame::*;
#[doc(inline)]
pub use types::*;

#[doc(inline)]
pub use device::*;
#[doc(inline)]
pub use error::*;
#[doc(inline)]
pub use stream::*;
