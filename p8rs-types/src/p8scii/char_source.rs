use core::borrow::Borrow;

/// Helper trait for types that can iterate over bytes
/// 
/// This trait automatically implemented for each type that implement [`IntoIterator`] where [`IntoIterator::Item`] implements [`Borrow<u8>`](Borrow),
/// which covers types such as `&[u8]`, `Iter<Item = u8>`, etc.
pub trait IntoByteIterator {
	fn into_iter(self) -> impl Iterator<Item = u8>;
}

impl<T, I> IntoByteIterator for T
where T: IntoIterator<Item = I>,
      I: Borrow<u8> {
	fn into_iter(self) -> impl Iterator<Item = u8> {
		self.into_iter().map(|b| *b.borrow())
	}
}
