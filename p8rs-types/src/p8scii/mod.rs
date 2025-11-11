//! Utilities for encoding, decoding and escaping P8SCII text
//! 
//! # Character Set
//! 
//! <script>
#![doc=include_str!("charTable.js")]
//! </script>
//!
//! _See Pico-8 Wiki for more info: <https://pico-8.fandom.com/wiki/P8SCII>_

mod char_source;
mod encoder;
mod decoder;
mod escape;
mod unescape;
mod utils;
mod fmt;

pub use char_source::IntoByteIterator;
pub use encoder::{from_char, from_iter, from_str, from_utf8, FromCharError};
pub use decoder::{to_char, to_iter};
pub use escape::escape;
pub use unescape::{unescape, unescape_in_place, UnescapeError};
pub use utils::{LossyIteratorEx, Printable};
pub use fmt::Display;

/// Character table mapping P8SCII characters to their Unicode counterparts.
/// 
/// This table does not contain Variation Selector-16 (U+FE0F) characters that Pico-8 typically inserts
/// after 131 Down key (⬇️), 139 Left key (⬅️), 142 O key (🅾️), 145 Right key (➡️) and 148 Up key (⬆️) characters.
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

/// Variation Selector-16 (U+FE0F) character
pub const VAR_SEL: char = '\u{FE0F}';

/// Returns true if a specific p8scii character requires Variation Selector-16 (U+FE0F) character in the utf-8 representation
/// 
/// That is: Down key (⬇️), 139 Left key (⬅️), 142 O key (🅾️), 145 Right key (➡️) and 148 Up key (⬆️)
pub fn requires_var_sel(char: u8) -> bool {
	matches!(char, 131 | 139 | 142 | 145 | 148) // ⬇️ ⬅️ 🅾️ ➡️ ⬆️
}
