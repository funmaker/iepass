#![allow(dead_code)]
//! An iterator adapter that allows you to efficiently peek the nth item of an iterator.
//!
//! Itermediate values are memoized and heap allocations are avoided when possible.
//! 
//! Based on https://github.com/zacharygolba/peek-nth
//!
//! ## Usage
//!
//! ```rust
//! extern crate p8rs_piccolo;
//!
//! use p8rs_piccolo::peek_nth::IteratorExt;
//!
//! fn main() {
//!     let mut iter = "Hello, world!".chars().peekable_nth::<8>();
//!
//!     assert_eq!(iter.peek_nth(4), Some(&'o'));
//!     assert_eq!(iter.peek_nth(3), Some(&'l'));
//!     assert_eq!(iter.peek_nth(2), Some(&'l'));
//!     assert_eq!(iter.peek_nth(1), Some(&'e'));
//!     assert_eq!(iter.peek_nth(0), Some(&'H'));
//!     assert_eq!(iter.peek_nth(7), Some(&'w'));
//!     assert_eq!(iter.collect::<String>(), "Hello, world!");
//! }
//!```

use std::iter::{DoubleEndedIterator, ExactSizeIterator};

use arrayvec::ArrayVec;

/// An iterator with a peek_nth() method that returns an optional reference to the nth element.
#[derive(Clone, Debug)]
pub struct PeekableNth<I, const SIZE: usize>
where
	I: Iterator,
{
	iter: I,
	next: ArrayVec<I::Item, SIZE>,
}

impl<I, const SIZE: usize> PeekableNth<I, SIZE>
where
	I: Iterator,
{
	/// Returns a reference to the next value without advancing the iterator.
	#[inline]
	pub fn peek(&mut self) -> Option<&I::Item> {
		self.peek_nth(0)
	}
	
	/// Returns a reference to the nth(n) value without advancing the iterator.
	#[inline]
	pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
		let length = self.next.len();
		let offset = n + 1;
		
		if offset > length {
			for _ in length..offset {
				self.next.push(self.iter.next()?);
			}
		}
		
		self.next.get(n)
	}
	
	/// Returns the number of elements buffered in the peek queue
	#[inline]
	pub fn peek_len(&mut self) -> usize {
		self.next.len()
	}
}

impl<I, const SIZE: usize> DoubleEndedIterator for PeekableNth<I, SIZE>
where
	I: DoubleEndedIterator,
{
	#[inline]
	fn next_back(&mut self) -> Option<I::Item> {
		match self.iter.next_back() {
			None if !self.next.is_empty() => self.next.pop(),
			option => option,
		}
	}
}

impl<I, const SIZE: usize> ExactSizeIterator for PeekableNth<I, SIZE>
where
	I: ExactSizeIterator,
{
	#[inline]
	fn len(&self) -> usize {
		self.iter.len()
	}
}

impl<I, const SIZE: usize> Iterator for PeekableNth<I, SIZE>
where
	I: Iterator,
{
	type Item = I::Item;
	
	#[inline]
	fn next(&mut self) -> Option<I::Item> {
		if self.next.is_empty() {
			self.iter.next()
		} else {
			Some(self.next.remove(0))
		}
	}
}

/// Adds a peekable_nth() method to types that implement [`std::iter::Iterator`].
///
/// [`std::iter::Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
pub trait IteratorExt: Iterator + Sized {
	fn peekable_nth<const SIZE: usize>(self) -> PeekableNth<Self, SIZE>;
}

impl<I> IteratorExt for I
where
	I: Iterator,
{
	#[inline]
	fn peekable_nth<const SIZE: usize>(self) -> PeekableNth<I, SIZE> {
		PeekableNth {
			iter: self,
			next: ArrayVec::new(),
		}
	}
}
