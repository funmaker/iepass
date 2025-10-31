use core::alloc::Allocator;
use core::ops::{Deref, DerefMut};
use alloc::alloc::Global;
use alloc::boxed::Box;
use bitflags::bitflags;
use crate::utils;

pub struct Memory<A: Allocator = Global> {
	inner: Box<[u8; 0x10000], A>,
}

impl<A: Allocator + Clone> Memory<A> {
	pub fn new(alloc: A) -> Memory<A> {
		let mut mem = Memory {
			inner: utils::new_zeroed_box_in(alloc),
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
	
	// TODO: use enums
	pub fn palette(&self, p_idx: u8) -> [u8; 16] {
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
	
	pub fn draw_state(&mut self) -> MemoryDrawState<'_, A> {
		MemoryDrawState(self)
	}
	
	pub fn hardware_state(&mut self) -> MemoryHardwareState<'_, A> {
		MemoryHardwareState(self)
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

impl<A: Allocator> Deref for Memory<A> {
	type Target = [u8; 0x10000];
	
	fn deref(&self) -> &Self::Target {
		&*self.inner
	}
}

impl<A: Allocator> DerefMut for Memory<A> {
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

pub struct MemoryDrawState<'a, A: Allocator + Clone>(&'a mut Memory<A>);

impl<'a, A: Allocator + Clone> MemoryDrawState<'a, A> {
	pub fn cursor_home_x(&mut self) -> &mut u8 {
		&mut self.0[0x5f24]
	}
	
	pub fn pen_color(&mut self) -> &mut u8 {
		&mut self.0[0x5f25]
	}
	
	pub fn cursor_position(&mut self) -> &mut [u8; 2] {
		(&mut self.0[0x5f26..=0x5f27]).try_into().unwrap()
	}
	
	/**
	 * [x_begin, y_begin, x_end, y_end]
	 */
	pub fn clip_rect(&mut self) -> &mut [u8; 4] {
		(&mut self.0[0x5f20..=0x5f23]).try_into().unwrap()
	}
	
	pub fn get_camera_position(&self) -> [i16; 2] {
		[ self.0.read_u16_le(0x5f28).cast_signed(), self.0.read_u16_le(0x5f2a).cast_signed() ]
	}
	
	pub fn set_camera_x(&mut self, value: i16) {
		self.0.write_u16_le(0x5f28, value.cast_unsigned());
	}
	
	pub fn set_camera_y(&mut self, value: i16) {
		self.0.write_u16_le(0x5f2a, value.cast_unsigned());
	}
}


bitflags! {
    pub struct PrintAttributeFlags: u8 {
        const ENABLE        = 1 << 0;
        const PADDING       = 1 << 1;
        const WIDE          = 1 << 2;
        const TALL          = 1 << 3;
        const SOLID_BG      = 1 << 4;
        const INVERT        = 1 << 5;
        const DOTTY         = 1 << 6;
        const CUSTOM_FONT   = 1 << 7;
    }
}


pub struct MemoryHardwareState<'a, A: Allocator + Clone>(&'a mut Memory<A>);

// 0x5f40..0x5f80
impl<'a, A: Allocator + Clone> MemoryHardwareState<'a, A> {
	pub fn get_print_defaults(&mut self) -> PrintAttributeFlags {
		PrintAttributeFlags::from_bits_truncate(self.0[0x5f58])
	}
	
	pub fn set_print_defaults(&mut self, flags: PrintAttributeFlags) {
		self.0[0x5f58] = flags.bits();
	}
}