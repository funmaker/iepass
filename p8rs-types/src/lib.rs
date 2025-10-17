#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]
#![feature(const_from)]
#![feature(const_ops)]
#![feature(const_option_ops)]
#![feature(const_slice_make_iter)]
#![feature(bigint_helper_methods)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(allocator_api)]

//! This crate provides an implementation of Pico-8's numeric type and string encoding (P8SCII).

pub mod p8scii;
pub mod p8num;
