use core::iter::Peekable;
use thiserror::Error;

use super::IntoByteIterator;

pub fn unescape(iter: impl IntoByteIterator) -> impl Iterator<Item = Result<u8, UnescapeError>> {
	Unescape::new(iter.into_iter())
}

pub fn unescape_in_place(buf: &mut [u8]) -> Result<usize, UnescapeError> {
	let buf = core::cell::RefCell::new(buf);
	
	let mut read_pos = 0;
	let mut write_pos = 0;
	let chars = Unescape::new(core::iter::from_fn(|| {
		let val = buf.borrow().get(read_pos).copied();
		read_pos += 1;
		val
	}));
	
	for ch in chars {
		buf.borrow_mut()[write_pos] = ch?;
		write_pos += 1;
	}
	
	Ok(write_pos)
}

struct Unescape<I: Iterator> {
	iter: Peekable<I>,
}

impl<I: Iterator<Item = u8>> Unescape<I> {
	fn new(iter: I) -> Self {
		Unescape { iter: iter.peekable() }
	}
	
	fn read_hex(&mut self) -> Result<u8, UnescapeError> {
		let hi = self.iter.next().and_then(hex_digit).ok_or(UnescapeError::InvalidEscapeSeq(b'x'))?;
		let lo = self.iter.next().and_then(hex_digit).ok_or(UnescapeError::InvalidEscapeSeq(b'x'))?;
		
		Ok(hi << 4 | lo)
	}
	
	fn read_decimal(&mut self, leading_char: u8) -> Result<u8, UnescapeError> {
		let mut value = (leading_char - b'0') as u16;
		
		if let Some(digit) = self.iter.next_if(|c| matches!(c, b'0' ..= b'9')) {
			value = value * 10 + (digit - b'0') as u16
		};
		
		if let Some(digit) = self.iter.next_if(|c| matches!(c, b'0' ..= b'9')) {
			value = value * 10 + (digit - b'0') as u16
		};
		
		if value > 255 {
			Err(UnescapeError::DecimalTooLarge(value))
		} else {
			Ok(value as u8)
		}
	}
}

impl<I: Iterator<Item = u8>> Iterator for Unescape<I> {
	type Item = Result<u8, UnescapeError>;
	
	fn next(&mut self) -> Option<Self::Item> {
		let next = self.iter.next()?;
		
		if next != b'\\' {
			return Some(Ok(next))
		}
		
		let parsed = match self.iter.next()? {
			b'*' => 1,
			b'#' => 2,
			b'-' => 3,
			b'|' => 4,
			b'+' => 5,
			b'^' => 6,
			b'a' => 7,
			b'b' => 8,
			b't' => 9,
			b'n' => 10,
			b'v' => 11,
			b'f' => 12,
			b'r' => 13,
			b'\\' => b'\\',
			b'\"' => b'\"',
			b'\'' => b'\'',
			b'\n' => b'\n',
			b'\r' => b'\r',
			b'x' => return Some(self.read_hex()),
			leading @ b'0' ..= b'9' => return Some(self.read_decimal(leading)),
			char => return Some(Err(UnescapeError::InvalidEscapeSeq(char))),
		};
		
		Some(Ok(parsed))
	}
}

fn hex_digit(digit: u8) -> Option<u8> {
	match digit {
		digit @ b'0'..=b'9' => Some(digit - b'0'),
		digit @ b'a'..=b'f' => Some(digit - b'a' + 10),
		digit @ b'A'..=b'F' => Some(digit - b'A' + 10),
		_ => None,
	}
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
pub enum UnescapeError {
	#[error("Decimal escape too large '\\{0}'")]
	DecimalTooLarge(u16),
	#[error("Invalid escape sequence near '\\{}'", super::to_char(*.0))]
	InvalidEscapeSeq(u8),
}
