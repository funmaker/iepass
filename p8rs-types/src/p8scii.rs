use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

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

pub struct P8String(Vec<u8>);

impl P8String {
	pub fn new() -> Self {
		P8String(Vec::new())
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
		P8String(Vec::with_capacity(capacity))
	}
	
	pub fn from_str(_value: &str) -> Result<Self, FromStrError> {
		unimplemented!()
	}
	
	pub fn from_str_lossy(_value: &str) -> Self {
		unimplemented!()
	}
}

impl From<Vec<u8>> for P8String {
	fn from(value: Vec<u8>) -> P8String {
		P8String(value)
	}
}

impl From<&[u8]> for P8String {
	fn from(value: &[u8]) -> P8String {
		P8String(value.to_vec())
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

pub enum FromStrError {
	UnexpectedChar(char, usize),
}
