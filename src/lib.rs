#![cfg_attr(not(feature = "std"), no_std)]

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
pub use frame::*;
pub use types::*;

#[cfg(feature = "std")]
pub use device::*;
#[cfg(feature = "std")]
pub use error::*;
#[cfg(feature = "std")]
pub use stream::*;
