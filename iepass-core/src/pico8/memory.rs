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
	}
	
	pub fn screen(&mut self) -> MemoryScreen<'_> {
		let mut base = self.base_addr_screen() as usize;
		if base > self.inner.len() - 0x2000 {
			base = 0x6000; // default if custom base would cause to wrap memory
		}
		MemoryScreen((&mut self.inner[base.. base+0x2000]).try_into().unwrap())
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
