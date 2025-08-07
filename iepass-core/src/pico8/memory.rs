use std::ops::{Deref, DerefMut};

pub struct Memory {
	inner: Box<[u8; 0x10000]>,
}

impl Memory {
	pub fn new() -> Memory {
		Memory {
			inner: vec![0; 0x10000].try_into().unwrap(),
		}
	}
	
	pub fn screen(&mut self) -> MemoryScreen<'_> {
		MemoryScreen((&mut self.inner[0x6000 .. 0x8000]).try_into().unwrap())
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
