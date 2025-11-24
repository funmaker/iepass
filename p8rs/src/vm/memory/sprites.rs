use core::ops::{Deref, DerefMut};

/// Usually 0x0000..=0x1fff
pub struct Sprites<'m>(#[allow(dead_code)] pub(super) &'m mut [u8; 0x2000]);

impl Sprites<'_> {
	pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
		if x >= 128 || y >= 128 { return; }
		
		let tuple = &mut self[y as usize * 64 + x as usize / 2];
		if x % 2 == 0 {
			*tuple = (*tuple & 0xF0) | (value & 0x0F);
		} else {
			*tuple = (*tuple & 0x0F) | ((value & 0x0F) << 4);
		}
	}
	
	pub fn get_pixel(&self, x: u8, y: u8) -> Option<u8> {
		if x >= 128 || y >= 128 { return None; }
		
		let tuple = self[y as usize * 64 + x as usize / 2];
		if x % 2 == 0 {
			Some(tuple & 0x0F)
		} else {
			Some(tuple >> 4)
		}
	}
}

impl Deref for Sprites<'_> {
	type Target = [u8; 0x2000];
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Sprites<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
