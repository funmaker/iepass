#![feature(pattern)]
#![feature(array_try_from_fn)]
#![feature(trim_prefix_suffix)]
#![feature(int_from_ascii)]
#![feature(iterator_try_collect)]
#![feature(array_try_map)]
#![feature(slice_as_array)]
#![feature(iter_array_chunks)]
#![feature(substr_range)]

pub const TMP_DIR: &str = "tmp";

pub mod utils;
pub mod log;
pub mod summary;

#[cfg(test)] mod tester;
#[cfg(test)] mod runner;

#[cfg(test)]
mod carts {
	include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
}
