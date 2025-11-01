#![cfg(test)]

pub mod tester;
mod carts {
	include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
}
