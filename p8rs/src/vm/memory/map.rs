use std::ops::{Deref, DerefMut};

/// Usually 0x0000..=0x1fff
#[allow(dead_code)] 
pub struct Map<'a> {
	pub(super) memory: &'a mut [u8; 0x2000],
	pub(super) width: usize,
	pub(super) height: usize,
}

impl Deref for Map<'_> {
	type Target = [u8; 0x2000];
	
	fn deref(&self) -> &Self::Target {
		&self.memory
	}
}

impl DerefMut for Map<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.memory
	}
}
