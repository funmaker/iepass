#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// MUST be the first module
mod fmt;
pub mod rle;
#[cfg(feature = "std")]
pub mod pico8;
pub mod colors;
