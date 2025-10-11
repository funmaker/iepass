#![cfg_attr(not(feature = "std"), no_std)]
#![feature(allocator_api)]
#![feature(substr_range)]
extern crate alloc;

// MUST be the first module
mod fmt;

pub mod rle;
pub mod pico8;
pub mod colors;
mod utils;
