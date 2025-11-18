use core::ops::{Deref, DerefMut};

/// Usually 0x6000..=0x7fff
pub struct Screen<'a>(pub(super) &'a mut [u8; 0x2000]);

impl Screen<'_> {
	fn get_addr(x: i16, y: i16) -> Result<(usize, bool), ()> {
		if x < 0 || y < 0 || x >= 128 || y >= 128 { return Err(()) }
		Ok((
			((x / 2) + y * 64) as usize,
			x & 1 == 0,
		))
	}
	
	pub fn get_pixel(&self, x: i16, y: i16) -> Result<u8, ()> {
		let (addr, high) = Self::get_addr(x, y)?;
		Ok(if high {
			self.0[addr] & 0xF
		} else {
			self.0[addr] >> 4
		})
	}
	
	pub fn set_pixel(&mut self, x: i16, y: i16, value: u8) -> Result<(), ()> {
		let (addr, high) = Self::get_addr(x, y)?;
		let old = self.0[addr];
		if high {
			self.0[addr] = (old & 0xF0) | (value & 0xF);
		} else {
			self.0[addr] = (value << 4) | (old & 0xF);
		}
		Ok(())
	}
	
	pub fn shift_up(&mut self, dy: u8, fill_color: u8) {
		let color = fill_color & 0xF;
		let color = color | (color << 4);
		
		if dy >= 128 {
			self.0.fill(color);
			return;
		}
		
		let dy = dy as usize;
		
		let copied_lines = 128 - dy;
		for line in 0..copied_lines {
			for x in 0..64 {
				self.0[line * 64 + x] = self.0[(dy + line) * 64 + x]
			}
		}
		
		for line in copied_lines..128 {
			for x in 0..64 {
				self.0[line * 64 + x] = color;
			}
		}
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
