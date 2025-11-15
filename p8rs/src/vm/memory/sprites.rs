use std::ops::{Deref, DerefMut};

/// Usually 0x0000..=0x1fff
pub struct Sprites<'a>(#[allow(dead_code)] pub(super) &'a mut [u8; 0x2000]);

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
