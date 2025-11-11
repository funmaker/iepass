use std::str::pattern::Pattern;
use thiserror::Error;

pub fn replace(mut source: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
	let mut out = Vec::with_capacity(source.len().saturating_sub(from.len()) + to.len());
	
	while let Some(pos) = source.windows(from.len())
	                            .position(|window| window == from) {
		out.extend_from_slice(&source[.. pos]);
		out.extend_from_slice(to);
		source = &source[pos + from.len() ..];
	}
	
	out.extend_from_slice(source);
	
	out
}

pub fn str_splitn_array<const N: usize, P: Pattern>(s: &str, pat: P) -> Option<[&str; N]> {
	let mut iter = s.splitn(N, pat);
	std::array::try_from_fn(|_| iter.next())
}

pub trait CollectArray {
	type Item;
	
	fn collect_array<const C: usize>(self) -> Result<[Self::Item; C], CollectArrayError>;
}

impl<T: Iterator> CollectArray for T {
	type Item = T::Item;
	
	fn collect_array<const C: usize>(mut self) -> Result<[Self::Item; C], CollectArrayError> {
		let ret = std::array::try_from_fn(|idx| self.next().ok_or(CollectArrayError::Underflow{ got: idx, expected: C }));
		if self.next().is_some() {
			Err(CollectArrayError::Overflow)
		} else {
			ret
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Error)]
pub enum CollectArrayError {
	#[error("Not enough elements (got {got}, expected {expected})")]
	Underflow{ got: usize, expected: usize },
	#[error("Too many elements")]
	Overflow,
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn replace_test() {
		assert_eq!(replace(b"Test lel kek lel wew", b"lel", b"banana"), b"Test banana kek banana wew")
	}
}