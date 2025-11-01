#![cfg(test)]
#![feature(pattern)]
#![feature(array_try_from_fn)]

const TMP_DIR: &str = "tmp";

mod tester;
mod utils;
mod runner;

mod carts {
	include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
}
