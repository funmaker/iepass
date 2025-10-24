use crate::p8scii::{IntoByteIterator, CHAR_MAP};

pub fn to_char(c: u8) -> char {
	CHAR_MAP[c as usize]
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

#[cfg(test)]
mod tests {
	extern crate alloc;
	
	use alloc::string::String;
	use super::*;
	
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
	fn test_to_iter() {
		assert_eq!(
			to_iter(b"The quick brown fox jumps over the lazy dog.").collect::<String>(),
			"The quick brown fox jumps over the lazy dog.",
		);
		assert_eq!(
			to_iter(&[185, 161, 28, 185, 161, 175, 165, 172, 154, 162, 30, 194, 29]).collect::<String>(),
			"みく、みくにしてあけ゛る。",
		);
		assert_eq!(
			to_iter(&[148, 131, 139, 145, 142, 151, 128, 129, 132, 152, 153]).collect::<String>(),
			"⬆️⬇️⬅️➡️🅾️❎█▒░▤▥",
		);
		assert_eq!(
			to_iter(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]).collect::<String>(),
			"\0¹²³⁴⁵⁶⁷⁸\t\nᵇᶜ\rᵉᶠ",
		);
	}
}
