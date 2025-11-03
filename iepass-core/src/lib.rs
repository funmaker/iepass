#![cfg_attr(not(feature = "std"), no_std)]
#![feature(allocator_api)]
#![feature(substr_range)]
#![feature(int_from_ascii)]
#![feature(slice_split_once)]
#![feature(type_alias_impl_trait)]
extern crate alloc;
extern crate core;

// MUST be the first module
mod fmt;

pub mod rle;
pub mod pico8;
pub mod colors;
mod utils;
