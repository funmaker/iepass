use super::Memory;

/// Usually 0x0000..=0x1fff
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Sprites {
	offset: u16,
}

impl Sprites {
	pub const WIDTH: u16 = 128;
	pub const HEIGHT: u16 = 128;
	
	pub(super) fn new(offset: u16) -> Sprites {
		Sprites { offset }
	}
	
	pub fn as_slice<'m>(&self, memory: &'m Memory) -> &'m [u8; 0x2000] {
		memory.const_slice::<0x2000>(self.offset)
	}
	
	pub fn as_slice_mut<'m>(&self, memory: &'m mut Memory) -> &'m mut [u8; 0x2000] {
		memory.const_slice_mut::<0x2000>(self.offset)
	}
	
	pub fn set_pixel(&self, memory: &mut Memory, x: u8, y: u8, value: u8) {
		if x >= 128 || y >= 128 { return; }
		
		let slice = self.as_slice_mut(memory);
		let tuple = &mut slice[y as usize * 64 + x as usize / 2];
		if x % 2 == 0 {
			*tuple = (*tuple & 0xF0) | (value & 0x0F);
		} else {
			*tuple = (*tuple & 0x0F) | ((value & 0x0F) << 4);
		}
	}
	
	pub fn get_pixel(&self, memory: &Memory, x: u8, y: u8) -> Option<u8> {
		if x >= 128 || y >= 128 { return None; }
		
		let slice = self.as_slice(memory);
		let tuple = slice[y as usize * 64 + x as usize / 2];
		if x % 2 == 0 {
			Some(tuple & 0x0F)
		} else {
			Some(tuple >> 4)
		}
	}
	
	pub fn sprite_pos(&self, idx: u8) -> [u8; 2] {
		[
			(idx % 16) * 8,
			(idx / 16) * 8,
		]
	}
}
