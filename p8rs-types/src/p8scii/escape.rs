use super::IntoByteIterator;

pub fn escape(iter: impl IntoByteIterator) -> impl Iterator<Item = u8> {
	Escape::new(iter.into_iter())
}

struct Escape<I> {
	iter: I,
	tail: &'static [u8],
}

impl<I: Iterator<Item = u8>> Escape<I> {
	fn new(iter: I) -> Self {
		Escape { iter, tail: &[] }
	}
}

impl<I: Iterator<Item = u8>> Iterator for Escape<I> {
	type Item = u8;
	
	fn next(&mut self) -> Option<Self::Item> {
		if let [char, rest @ ..] = self.tail {
			self.tail = rest;
			Some(*char)
		} else {
			let next = self.iter.next()?;
			
			self.tail = match next {
				0 => b"0",
				1 => b"*",
				2 => b"#",
				3 => b"-",
				4 => b"|",
				5 => b"+",
				6 => b"^",
				7 => b"a",
				8 => b"b",
				9 => b"t",
				10 => b"n",
				11 => b"v",
				12 => b"f",
				13 => b"r",
				14 => b"14",
				15 => b"15",
				b'"' => b"\"",
				b'\'' => b"\'",
				b'\\' => b"\\",
				_ => b"",
			};
			
			if self.tail.is_empty() {
				Some(next)
			} else {
				Some(b'\\')
			}
		}
	}
}
