use core::fmt::Debug;
use core::iter::Peekable;
use crate::vm::cart::CartLoadError;

type HeadersIter<'a> = impl Iterator<Item = &'a [u8]> + Debug;

#[derive(Debug)]
pub struct SectionIterator<'a> {
	cart: &'a [u8],
	headers_iter: Peekable<HeadersIter<'a>>,
}

impl<'a> SectionIterator<'a> {
	#[define_opaque(HeadersIter)]
	pub fn new(cart: &'a [u8]) -> Result<SectionIterator<'a>, CartLoadError> {
		let headers_iter = cart.split(|x: &u8| *x == b'\n' || *x == b'\r')
		                       .filter(|line| line.starts_with(b"__") && line.ends_with(b"__"))
		                       .peekable();
		
		if !cart.starts_with(b"pico-8 cartridge") {
			info!("SectionIterator: Invalid file header");
			return Err(CartLoadError::InvalidHeader);
		}
		
		Ok(SectionIterator {
			cart,
			headers_iter,
		})
	}
}

impl<'a> Iterator for SectionIterator<'a> {
	type Item = (&'a [u8], &'a [u8]);
	
	fn next(&mut self) -> Option<Self::Item> {
		let header = self.headers_iter.next()?;
		let next_header = self.headers_iter.peek();
		
		let body_start = self.cart.subslice_range(header).unwrap().end;
		let body_end = next_header.and_then(|header| self.cart.subslice_range(header))
		                          .map(|range| range.start)
		                          .unwrap_or(self.cart.len());
		
		Some((
			header,
			&self.cart[body_start..body_end],
		))
	}
}
