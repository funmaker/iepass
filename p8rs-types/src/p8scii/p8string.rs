use alloc::alloc::Global;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::ops::{Deref, DerefMut};

use super::{from_str, FromStrError};

#[derive(Debug, Clone, Hash)]
pub struct P8String<A: Allocator = Global>(Vec<u8, A>);

impl P8String<Global> {
	pub fn new() -> Self {
		P8String::new_in(Global)
	}
	
	pub fn from_str(data: &str) -> Result<Self, FromStrError> {
		P8String::from_str_in(data, Global)
	}
}

impl<A: Allocator> P8String<A> {
	pub fn new_in(alloc: A) -> Self {
		P8String(Vec::new_in(alloc))
	}
	
	pub fn from_str_in(data: &str, alloc: A) -> Result<Self, FromStrError> {
		let mut this = P8String(Vec::with_capacity_in(data.len(), alloc));
		
		this.extend_from_str(data)?;
		
		Ok(this)
	}
	
	pub fn to_vec(self) -> Vec<u8, A> {
		self.0
	}
}

impl <A: Allocator> P8String<A> {
	fn extend_from_str(&mut self, data: &str) -> Result<(), FromStrError> {
		let original_len = self.0.len();
		self.0.reserve(data.len());
		
		for result in from_str(data) {
			match result {
				Ok(char) => self.0.push(char),
				Err(err) => {
					self.0.truncate(original_len);
					return Err(err);
				}
			}
		}
		
		Ok(())
	}
}

impl<A: Allocator> From<Vec<u8, A>> for P8String<A> {
	fn from(value: Vec<u8, A>) -> Self {
		P8String(value)
	}
}

impl Deref for P8String {
	type Target = Vec<u8>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for P8String {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<AL: Allocator, AR: Allocator> PartialEq<P8String<AR>> for P8String<AL> {
	fn eq(&self, other: &P8String<AR>) -> bool {
		self.0.eq(&other.0)
	}
}

impl<A: Allocator> Eq for P8String<A> {}

impl<A: Allocator> PartialEq<[u8]> for P8String<A> {
	fn eq(&self, other: &[u8]) -> bool {
		self.0.eq(other)
	}
}

impl<A: Allocator, const S: usize> PartialEq<[u8; S]> for P8String<A> {
	fn eq(&self, other: &[u8; S]) -> bool {
		self.0.eq(other)
	}
}

impl<A: Allocator, const S: usize> PartialEq<&[u8; S]> for P8String<A> {
	fn eq(&self, other: &&[u8; S]) -> bool {
		self.0.eq(other)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_from_str() {
		assert_eq!(
			P8String::from_str("The quick brown fox jumps over the lazy dog.").unwrap(),
			b"The quick brown fox jumps over the lazy dog.",
		);
		assert_eq!(
			P8String::from_str("0123456789𓂸"),
			Err(FromStrError { char: '𓂸', pos: 10 }),
		);
	}
}
