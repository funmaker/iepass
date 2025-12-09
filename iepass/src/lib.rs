#![no_std]
#![allow(incomplete_features)]
#![feature(never_type)]
#![feature(iter_array_chunks)]
#![feature(generic_const_exprs)]

#[macro_use] extern crate p8rs_log;
extern crate alloc;

pub mod calib;
pub mod peripherials;
pub mod tasks;
pub mod utils;
