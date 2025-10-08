use alloc::string::String;
use thiserror::Error;

mod p8string;

pub use p8string::P8String;

pub const CHAR_MAP: [char; 256] = [
	// Control codes
	'\0', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '\t', '\n', 'ᵇ', 'ᶜ', '\r', 'ᵉ', 'ᶠ',
	// Symbols
	'▮', '■', '□', '⁙', '⁘', '‖', '◀', '▶', '「', '」', '¥', '•', '、', '。',
	// Japanese punctuation
	'゛', '゜',
	// ASCII Characters
	' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
	':', ';', '<', '=', '>', '?', '@',
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
	'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
	'[', '\\', ']', '^', '_', '`',
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
	'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
	'{', '|', '}', '~',
	// Symbols
	'○', '█', '▒', '🐱', '⬇', '░', '✽', '●', '♥', '☉', '웃', '⌂', '⬅', '😐',
	'♪', '🅾', '◆', '…', '➡', '★', '⧗', '⬆', 'ˇ', '∧', '❎', '▤', '▥',
	// Hiragana
	'あ', 'い', 'う', 'え', 'お',
	'か', 'き', 'く', 'け', 'こ',
	'さ', 'し', 'す', 'せ', 'そ',
	'た', 'ち', 'つ', 'て', 'と',
	'な', 'に', 'ぬ', 'ね', 'の',
	'は', 'ひ', 'ふ', 'へ', 'ほ',
	'ま', 'み', 'む', 'め', 'も',
	'や',      'ゆ',      'よ',
	'ら', 'り', 'る', 'れ', 'ろ',
	'わ',                'を',
	'ん', 'っ',
	'ゃ',      'ゅ',      'ょ',
	// Katakana
	'ア', 'イ', 'ウ', 'エ', 'オ',
	'カ', 'キ', 'ク', 'ケ', 'コ',
	'サ', 'シ', 'ス', 'セ', 'ソ',
	'タ', 'チ', 'ツ', 'テ', 'ト',
	'ナ', 'ニ', 'ヌ', 'ネ', 'ノ',
	'ハ', 'ヒ', 'フ', 'ヘ', 'ホ',
	'マ', 'ミ', 'ム', 'メ', 'モ',
	'ヤ',      'ユ',      'ヨ',
	'ラ', 'リ', 'ル', 'レ', 'ロ',
	'ワ',                'ヲ',
	'ン', 'ッ',
	'ャ',      'ュ',      'ョ',
	// Symbols
	'◜', '◝'
];

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

pub fn to_char(c: u8) -> char {
	CHAR_MAP[c as usize]
}

pub fn to_iter<'a>(iter: impl IntoIterator<Item = &'a u8> + 'a) -> impl Iterator<Item = char> + 'a {
	IntoP8Scii {
		iter: iter.into_iter(),
		variant_sel: false,
	}
}

pub fn to_string<'a>(iter: impl IntoIterator<Item = &'a u8> + 'a) -> String {
	to_iter(iter).collect()
}

struct IntoP8Scii<'a, T: Iterator<Item = &'a u8>> {
	iter: T,
	variant_sel: bool,
}

impl<'a, T: Iterator<Item = &'a u8>> Iterator for IntoP8Scii<'a, T> {
	type Item = char;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.variant_sel {
			self.variant_sel = false;
			Some('\u{FE0F}')
		} else {
			let next = *self.iter.next()?;
			self.variant_sel = matches!(next, 131 | 139 | 145 | 148 | 142);
			Some(to_char(next))
		}
	}
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
#[error("Unexpected character {char} ({char:?})")]
pub struct FromCharError {
	pub char: char,
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
#[error("Unexpected character {char} ({char:?}) at {pos} character")]
pub struct FromStrError {
	pub pos: usize,
	pub char: char,
}

#[cfg(test)]
mod tests {
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
	
	#[test]
	fn test_to_string() {
		assert_eq!(
			to_string(b"The quick brown fox jumps over the lazy dog."),
			"The quick brown fox jumps over the lazy dog.",
		);
		assert_eq!(
			to_string(&[185, 161, 28, 185, 161, 175, 165, 172, 154, 162, 30, 194, 29]),
			"みく、みくにしてあけ゛る。",
		);
		assert_eq!(
			to_string(&[148, 131, 139, 145, 142, 151, 128, 129, 132, 152, 153]),
			"⬆️⬇️⬅️➡️🅾️❎█▒░▤▥",
		);
		assert_eq!(
			to_string(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
			"\0¹²³⁴⁵⁶⁷⁸\t\nᵇᶜ\rᵉᶠ",
		);
	}
}
