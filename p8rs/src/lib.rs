#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![feature(allocator_api)]
#![feature(substr_range)]
#![feature(int_from_ascii)]
#![feature(slice_split_once)]
#![feature(type_alias_impl_trait)]
#![feature(range_into_bounds)]
#![feature(range_bounds_is_empty)]
#![feature(iter_array_chunks)]
#[macro_use] extern crate p8rs_log;
extern crate alloc;
extern crate core;

pub use p8rs_log::*;
pub use p8rs_macros as macros;
pub use p8rs_piccolo as piccolo;
pub use p8rs_types as types;

pub mod rle;
pub mod vm;
pub mod colors;
mod utils;
