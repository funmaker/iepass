use arrayvec::ArrayVec;
use crate::p8scii::encoder::FromUtf8Error;

pub trait LossyIteratorEx {
	fn lossy(self) -> impl Iterator<Item = u8>;
}

impl<T, E> LossyIteratorEx for T
where T: Iterator<Item = Result<u8, E>>,
      E: Into<FromUtf8Error> {
	fn lossy(self) -> impl Iterator<Item = u8> {
		self.flat_map(|res| {
			match res.map_err(Into::into) {
				Ok(byte) => ArrayVec::from_iter([byte]),
				Err(FromUtf8Error::Byte(byte)) => ArrayVec::from_iter([byte]),
				Err(FromUtf8Error::Char(char)) => {
					let mut buf = ArrayVec::from([0; 4]);
					let len = char.encode_utf8(&mut buf).as_bytes().len();
					buf.truncate(len);
					buf
				},
			}
		})
	}
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;
	use super::*;
	use crate::p8scii;
	
	#[test]
	fn test_lossy() {
		assert_eq!(
			p8scii::from_utf8("The quick brown fox jumps over the lazy dog.".as_bytes()).lossy().collect::<Vec<_>>(),
			b"The quick brown fox jumps over the lazy dog.",
		);
		assert_eq!(
			p8scii::from_utf8("みく、みくにしてあけ゛る。".as_bytes()).lossy().collect::<Vec<_>>(),
			[185, 161, 28, 185, 161, 175, 165, 172, 154, 162, 30, 194, 29],
		);
		assert_eq!(
			p8scii::from_utf8("012ą345好abc𓂸def".as_bytes()).lossy().collect::<Vec<_>>(),
			[
				b'0', b'1', b'2', 196, 133,
				b'3', b'4', b'5', 229, 165, 189,
				b'a', b'b', b'c', 240, 147, 130, 184,
				b'd', b'e', b'f'
			],
		);
		assert_eq!(
			p8scii::from_utf8(&[0, 1, 2, 3, 4, 5, 250, 251, 252, 253, 254, 255]).lossy().collect::<Vec<_>>(),
			[0, 1, 2, 3, 4, 5, 250, 251, 252, 253, 254, 255],
		);
	}
}
