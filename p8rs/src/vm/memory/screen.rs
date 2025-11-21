use core::ops::{Deref, DerefMut};

/// Usually 0x6000..=0x7fff
pub struct Screen<'m>(pub(super) &'m mut [u8; 0x2000]);

impl Screen<'_> {
	fn get_addr(x: u8, y: u8) -> Option<(usize, bool)> {
		if x >= 128 || y >= 128 { return None }
		Some((
			(x as usize / 2) + y as usize * 64,
			x & 1 == 0,
		))
	}
	
	pub fn get_pixel(&self, x: u8, y: u8) -> Option<u8> {
		let (addr, high) = Self::get_addr(x, y)?;
		if high {
			Some(self.0[addr] & 0xF)
		} else {
			Some(self.0[addr] >> 4)
		}
	}
	
	pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) -> bool {
		let Some((addr, high)) = Self::get_addr(x, y) else { return false };
		
		let old = self.0[addr];
		if high {
			self.0[addr] = (old & 0xF0) | (value & 0xF);
		} else {
			self.0[addr] = (value << 4) | (old & 0xF);
		}
		
		true
	}
	
	pub fn shift_up(&mut self, dy: u8, fill_color: u8) {
		if dy == 0 { return }
		
		let start = dy.min(128) as usize * 64;
		let end = self.len();
		self.copy_within(start..end, 0);
		self[end-start..].fill(fill_color);
	}
}

impl Deref for Screen<'_> {
	type Target = [u8; 0x2000];
	
	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

impl DerefMut for Screen<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.0
	}
}
