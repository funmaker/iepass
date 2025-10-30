use thiserror::Error;

use super::CHAR_MAP;

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

pub fn from_iter(data: impl IntoIterator<Item = char>) -> impl Iterator<Item = Result<u8, FromCharError>> {
	data.into_iter()
	    .map(from_char)
		.flat_map(Result::transpose)
}

pub fn from_str(data: &str) -> impl Iterator<Item = Result<u8, FromCharError>> {
	from_iter(data.chars())
}

pub fn from_utf8(data: &[u8]) -> impl Iterator<Item = Result<u8, FromUtf8Error>> + '_ {
	DecodeUtf8::new(data)
		.flat_map(|res| match res.map(from_char) {
			Ok(Ok(Some(char))) => Some(Ok(char)),
			Ok(Ok(None)) => None,
			Ok(Err(err)) => Some(Err(err.into())),
			Err(byte) => Some(Err(FromUtf8Error::Byte(byte))),
		})
}

struct DecodeUtf8<'a> {
	valid: &'a str,
	invalid: &'a [u8],
	remaining: &'a [u8],
}

impl<'a> DecodeUtf8<'a> {
	fn new(data: &'a [u8]) -> Self {
		Self {
			valid: "",
			invalid: &[],
			remaining: data,
		}
	}
}

impl<'a> Iterator for DecodeUtf8<'a> {
	type Item = Result<char, u8>;
	
	fn next(&mut self) -> Option<Self::Item> {
		if let Some((char, rest)) = split_first_char(self.valid) {
			self.valid = rest;
			Some(Ok(char))
		} else if let Some((&byte, rest)) = self.invalid.split_first() {
			self.invalid = rest;
			Some(Err(byte))
		} else if !self.remaining.is_empty() {
			match str::from_utf8(self.remaining) {
				Ok(valid) => {
					self.remaining = &[];
					self.valid = valid;
				}
				Err(err) => {
					let (valid, rest) = self.remaining.split_at(err.valid_up_to());
					self.valid = str::from_utf8(valid).unwrap();
					(self.invalid, self.remaining) = rest.split_at(err.error_len().unwrap_or(rest.len()));
				}
			}
			self.next()
		} else {
			None
		}
	}
}

fn split_first_char(string: &str) -> Option<(char, &str)> {
	let mut chars = string.chars();
	chars.next().map(|ch| (ch, chars.as_str()))
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
#[error("Unexpected character {char} ({char:?})")]
/// The error type returned when an unexpected character is encountered.
pub struct FromCharError {
	pub char: char,
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
/// The error type returned when an unexpected character or invalid utf8 sequence is encountered.
pub enum FromUtf8Error {
	#[error("Unexpected character {0} ({0:?})")]
	Char(char),
	#[error("Unexpected byte {0} ({0:?})")]
	Byte(u8),
}

impl From<FromCharError> for FromUtf8Error {
	fn from(value: FromCharError) -> Self {
		FromUtf8Error::Char(value.char)
	}
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
			from_str("012345𓂸abcdef").collect::<Result<Vec<_>, _>>(),
			Err(FromCharError { char: '𓂸' }),
		);
	}
	
	#[test]
	fn test_from_utf8() {
		assert_eq!(
			from_utf8("The quick brown fox jumps over the lazy dog.".as_bytes()).collect::<Result<Vec<_>, _>>().unwrap(),
			b"The quick brown fox jumps over the lazy dog.",
		);
		assert_eq!(
			from_utf8("みく、みくにしてあけ゛る。".as_bytes()).collect::<Result<Vec<_>, _>>().unwrap(),
			[185, 161, 28, 185, 161, 175, 165, 172, 154, 162, 30, 194, 29],
		);
		assert_eq!(
			from_utf8("⬆️⬇️⬅️➡️🅾️❎█▒░▤▥".as_bytes()).collect::<Result<Vec<_>, _>>().unwrap(),
			[148, 131, 139, 145, 142, 151, 128, 129, 132, 152, 153],
		);
		assert_eq!(
			from_utf8("\0¹²³⁴⁵⁶⁷⁸\t\nᵇᶜ\rᵉᶠ".as_bytes()).collect::<Result<Vec<_>, _>>().unwrap(),
			[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
		);
		assert_eq!(
			from_utf8("012345𓂸abcdef".as_bytes()).collect::<Result<Vec<_>, _>>(),
			Err(FromUtf8Error::Char('𓂸')),
		);
		assert_eq!(
			from_utf8(&[254, 255]).collect::<Vec<_>>(),
			[
				Err(FromUtf8Error::Byte(254)),
				Err(FromUtf8Error::Byte(255)),
			],
		);
	}
}
