use core::alloc::Allocator;
use alloc::boxed::Box;

pub fn new_zeroed_box_in<A: Allocator, const N: usize>(alloc: A) -> Box<[u8; N], A> {
	let ret = Box::new_zeroed_in(alloc);
	unsafe { ret.assume_init() } // SAFETY: This is just a zeroed u8 array.
}
