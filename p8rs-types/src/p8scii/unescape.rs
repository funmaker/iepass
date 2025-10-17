use core::iter::Peekable;
use thiserror::Error;

use super::IntoByteIterator;

pub fn unescape(iter: impl IntoByteIterator) -> impl Iterator<Item = Result<u8, UnescapeError>> {
	Unescape::new(iter.into_iter())
}

struct Unescape<I: Iterator> {
	iter: Peekable<I>,
}

impl<I: Iterator<Item = u8>> Unescape<I> {
	fn new(iter: I) -> Self {
		Unescape { iter: iter.peekable() }
	}
}

impl<I: Iterator<Item = u8>> Iterator for Unescape<I> {
	type Item = Result<u8, UnescapeError>;
	
	fn next(&mut self) -> Option<Self::Item> {
		let next = self.iter.next()?;
		
		if next != b'\\' {
			return Some(Ok(next))
		}
		
		let mut value = match self.iter.next()? {
			b'*' => return Some(Ok(1)),
			b'#' => return Some(Ok(2)),
			b'-' => return Some(Ok(3)),
			b'|' => return Some(Ok(4)),
			b'+' => return Some(Ok(5)),
			b'^' => return Some(Ok(6)),
			b'a' => return Some(Ok(7)),
			b'b' => return Some(Ok(8)),
			b't' => return Some(Ok(9)),
			b'n' => return Some(Ok(10)),
			b'v' => return Some(Ok(11)),
			b'f' => return Some(Ok(12)),
			b'r' => return Some(Ok(13)),
			b'\"' => return Some(Ok(b'"')),
			b'\'' => return Some(Ok(b'\'')),
			digit @ b'0' ..= b'9' => (digit - b'0') as u16,
			char => return Some(Err(UnescapeError::InvalidEscapeSeq(char))),
		};
		
		if let Some(digit) = self.iter.next_if(|c| matches!(c, b'0' ..= b'9')) {
			value = value * 10 + (digit - b'0') as u16
		};
		
		if let Some(digit) = self.iter.next_if(|c| matches!(c, b'0' ..= b'9')) {
			value = value * 10 + (digit - b'0') as u16
		};
		
		if value > 255 {
			Some(Err(UnescapeError::DecimalTooLarge(value)))
		} else {
			Some(Ok(value as u8))
		}
	}
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
pub enum UnescapeError {
	#[error("Decimal escape too large '\\{0}'")]
	DecimalTooLarge(u16),
	#[error("Invalid escape sequence near '\\{}'", super::to_char(*.0))]
	InvalidEscapeSeq(u8),
}
