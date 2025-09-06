use std::ops::{Deref, DerefMut};

pub struct Memory {
	inner: Box<[u8; 0x10000]>,
}

impl Memory {
	pub fn new() -> Memory {
		let mut mem = Memory {
			inner: vec![0u8; 0x10000].try_into().unwrap(),
		};
		
		mem.reset();
		mem
	}
	
	pub fn reset(&mut self) {
		self.inner[0x5F55] = 0x60; // default screen mapping
		self.inner[0x5F56] = 0x20; // default map mapping
		self.inner[0x5F57] = 128; // default map size
		self.inner[0x5f22] = 128; // default clip x_end
		self.inner[0x5f23] = 128; // default clip y_end
		
		for i in 0..16 {
			self.inner[0x5F00 + i] = i as u8;
			self.inner[0x5F10 + i] = i as u8;
		}
		self.inner[0x5F00] = 0x10; // transparent
	}
	
	pub fn palette(&self, p_idx: u8) -> &[u8; 16] {
		let base = self.base_addr_palette(p_idx) as usize;
		self.inner[base..base+16].try_into().unwrap()
	}
	
	pub fn screen(&mut self) -> MemoryScreen<'_> {
		let mut base = self.base_addr_screen() as usize;
		if base > self.inner.len() - 0x2000 {
			base = 0x6000; // default if custom base would cause to wrap memory
		}
		MemoryScreen((&mut self.inner[base.. base+0x2000]).try_into().unwrap())
	}
	
	pub fn base_addr_palette(&self, p_idx: u8) -> u16 {
		match p_idx {
			0 => 0x5f00,
			1 => 0x5f10,
			2 => 0x5f60,
			_ => panic!("Invalid palette index: {}", p_idx),
		}
	}
	
	pub fn base_addr_gfx(&self) -> u16 {
		(self.inner[0x5F54] as u16).wrapping_shl(8)
	}
	
	pub fn base_addr_screen(&self) -> u16 {
		(self.inner[0x5F55] as u16).wrapping_shl(8)
	
	}
	pub fn base_addr_map(&self) -> u16 {
		(self.inner[0x5F56] as u16).wrapping_shl(8)
	}
	
	pub fn read_u16_le(&self, addr: u16) -> u16 {
		assert!(addr < 0xffff, "Address out of bounds");
		let addr = addr as usize;
		((self.inner[addr] as u16) << 8) | self.inner[addr + 1] as u16
	}
	
	pub fn write_u16_le(&mut self, addr: u16, val: u16) {
		assert!(addr < 0xffff, "Address out of bounds");
		let addr = addr as usize;
		self.inner[addr] = (val >> 8) as u8;
		self.inner[addr + 1] = val as u8;
	}
}

impl Deref for Memory {
	type Target = [u8; 0x10000];
	
	fn deref(&self) -> &Self::Target {
		&*self.inner
	}
}

impl DerefMut for Memory {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.inner
	}
}

pub struct MemoryScreen<'a>(&'a mut [u8; 0x2000]);

impl<'a> Deref for MemoryScreen<'a> {
	type Target = [u8; 0x2000];
	
	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

impl<'a> DerefMut for MemoryScreen<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.0
	}
}
