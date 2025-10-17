use thiserror::Error;

use super::{CHAR_MAP, IntoByteIterator};

pub fn from_char(c: char) -> Result<Option<u8>, FromCharError> {
	if let ' '..'~' | '\n' | '\t' | '\r' | '\0' = c {
		Ok(Some(c as u8))
	} else if c == '\u{FE0F}' {
		Ok(None)
	} else if let Some(pos) = CHAR_MAP.iter().position(|&ch| ch == c) {
		Ok(Some(pos as u8))
	} else {
		Err(FromCharError { char: c })
	}
}

pub fn to_char(c: u8) -> char {
	CHAR_MAP[c as usize]
}

pub fn from_iter(data: impl IntoIterator<Item = char>) -> impl Iterator<Item = Result<u8, FromStrError>> {
	data.into_iter()
	    .map(from_char)
	    .enumerate()
	    .flat_map(|(pos, res)|
		    res.map_err(|err| FromStrError { pos, char: err.char })
		       .transpose())
}

pub fn from_str(data: &str) -> impl Iterator<Item = Result<u8, FromStrError>> {
	from_iter(data.chars())
}

pub fn to_iter(iter: impl IntoByteIterator) -> impl Iterator<Item = char> {
	IntoChars::new(iter.into_iter())
}

struct IntoChars<I> {
	iter: I,
	variant_sel: bool,
}

impl<I: Iterator<Item = u8>> IntoChars<I> {
	fn new(iter: I) -> Self {
		IntoChars {
			iter,
			variant_sel: false,
		}
	}
}

impl<T: Iterator<Item = u8>> Iterator for IntoChars<T> {
	type Item = char;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.variant_sel {
			self.variant_sel = false;
			Some('\u{FE0F}')
		} else {
			let next = self.iter.next()?;
			self.variant_sel = matches!(next, 131 | 139 | 145 | 148 | 142);
			Some(to_char(next))
		}
	}
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
#[error("Unexpected character {char} ({char:?})")]
/// The error type returned when a [from_char] fails.
pub struct FromCharError {
	pub char: char,
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
#[error("Unexpected character {char} ({char:?}) at {pos} character")]
/// The error type returned when a [from_str] or [from_iter] fails.
pub struct FromStrError {
	pub pos: usize,
	pub char: char,
}

#[cfg(test)]
mod tests {
	extern crate alloc;
	
	use alloc::vec::Vec;
	use super::*;
	
	#[test]
	fn test_from_char() {
		assert_eq!(from_char('a'), Ok(Some(b'a')));
		assert_eq!(from_char(' '), Ok(Some(b' ')));
		assert_eq!(from_char('\n'), Ok(Some(b'\n')));
		assert_eq!(from_char('あ'), Ok(Some(154)));
		assert_eq!(from_char('ア'), Ok(Some(204)));
		assert_eq!(from_char('◝'), Ok(Some(255)));
		assert_eq!(from_char('\0'), Ok(Some(0)));
		assert_eq!(from_char('ᶠ'), Ok(Some(15)));
		assert_eq!(from_char('\u{FE0F}'), Ok(None));
		assert_eq!(from_char('𓂸'), Err(FromCharError { char: '𓂸' }));
	}
	
	#[test]
	fn test_to_char() {
		assert_eq!(to_char(b'a'), 'a');
		assert_eq!(to_char(b' '), ' ');
		assert_eq!(to_char(b'\n'), '\n');
		assert_eq!(to_char(154), 'あ');
		assert_eq!(to_char(204), 'ア');
		assert_eq!(to_char(255), '◝');
		assert_eq!(to_char(0), '\0');
		assert_eq!(to_char(15), 'ᶠ');
	}
	
	#[test]
	fn test_from_str() {
		assert_eq!(
			from_str("The quick brown fox jumps over the lazy dog.").collect::<Result<Vec<_>, _>>().unwrap(),
			b"The quick brown fox jumps over the lazy dog.",
		);
		assert_eq!(
			from_str("みく、みくにしてあけ゛る。").collect::<Result<Vec<_>, _>>().unwrap(),
			[185, 161, 28, 185, 161, 175, 165, 172, 154, 162, 30, 194, 29],
		);
		assert_eq!(
			from_str("⬆️⬇️⬅️➡️🅾️❎█▒░▤▥").collect::<Result<Vec<_>, _>>().unwrap(),
			[148, 131, 139, 145, 142, 151, 128, 129, 132, 152, 153],
		);
		assert_eq!(
			from_str("\0¹²³⁴⁵⁶⁷⁸\t\nᵇᶜ\rᵉᶠ").collect::<Result<Vec<_>, _>>().unwrap(),
			[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
		);
		assert_eq!(
			from_str("0123456789𓂸").collect::<Result<Vec<_>, _>>(),
			Err(FromStrError { char: '𓂸', pos: 10 }),
		);
	}
}
