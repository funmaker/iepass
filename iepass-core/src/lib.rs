#![cfg_attr(all(not(test), not(feature = "std")), no_std)]


pub mod rle;
#[cfg(feature = "std")]
pub mod pico8;
pub mod colors;
