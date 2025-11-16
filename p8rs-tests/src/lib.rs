#![cfg(test)]
#![feature(pattern)]
#![feature(array_try_from_fn)]
#![feature(trim_prefix_suffix)]
#![feature(int_from_ascii)]
#![feature(iterator_try_collect)]
#![feature(array_try_map)]
#![feature(slice_as_array)]
#![feature(iter_array_chunks)]

const TMP_DIR: &str = "tmp";

mod tester;
mod utils;
mod runner;
mod log;

mod carts {
	include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
}
