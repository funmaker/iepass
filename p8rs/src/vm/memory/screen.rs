use super::Memory;

/// Usually 0x6000..=0x7fff
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Screen {
	offset: u16,
}

impl Screen {
	pub(super) fn new(offset: u16) -> Screen {
		Screen { offset }
	}
	
	pub fn as_slice<'m>(&self, memory: &'m Memory) -> &'m [u8; 0x2000] {
		memory.const_slice::<0x2000>(self.offset)
	}
	
	pub fn as_slice_mut<'m>(&self, memory: &'m mut Memory) -> &'m mut [u8; 0x2000] {
		memory.const_slice_mut::<0x2000>(self.offset)
	}
	
	pub fn set_pixel(&mut self, memory: &mut Memory, x: u8, y: u8, value: u8) {
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
	
	pub fn shift_up(&mut self, memory: &mut Memory, dy: u8, fill_color: u8) {
		if dy == 0 { return }
		
		let slice = self.as_slice_mut(memory);
		let start = dy.min(128) as usize * 64;
		let end = slice.len();
		slice.copy_within(start..end, 0);
		slice[end-start..].fill(fill_color);
	}
}

