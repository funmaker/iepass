use core::alloc::Allocator;
use alloc::boxed::Box;
use core::num::NonZeroU8;

pub fn new_zeroed_box_in<A: Allocator, const N: usize>(alloc: A) -> Box<[u8; N], A> {
	let ret = Box::new_zeroed_in(alloc);
	unsafe { ret.assume_init() } // SAFETY: This is just a zeroed u8 array.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NonZeroNibble(NonZeroU8);

impl NonZeroNibble {
	pub fn new(nibble: u8) -> Option<NonZeroNibble> {
		if nibble >= 16 {
			None
		} else {
			Some(NonZeroNibble(NonZeroU8::new(nibble)?))
		}
	}
	
	pub fn get(self) -> u8 {
		self.0.get()
	}
}
