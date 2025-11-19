use core::ops::{Deref, DerefMut, Index, IndexMut};

/// 0x3000..=0x30ff
pub struct SpriteFlags<'m>(#[allow(dead_code)] pub(super) &'m mut [u8; 256]);

impl Deref for SpriteFlags<'_> {
	type Target = [u8; 256];
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for SpriteFlags<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Index<u8> for SpriteFlags<'_> {
	type Output = u8;
	
	fn index(&self, index: u8) -> &Self::Output {
		&self.0[index as usize]
	}
}

impl IndexMut<u8> for SpriteFlags<'_> {
	fn index_mut(&mut self, index: u8) -> &mut Self::Output {
		&mut self.0[index as usize]
	}
}
