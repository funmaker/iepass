use core::alloc::Allocator;
use core::ops::{Deref, DerefMut};
use alloc::alloc::Global;
use alloc::boxed::Box;
use p8rs_types::p8num::P8Num;
use bitflags::bitflags;
use crate::utils;

pub struct Memory<A: Allocator = Global> {
	inner: Box<[u8; 0x10000], A>,
}

impl<A: Allocator> Memory<A> {
	pub fn new(alloc: A) -> Memory<A> {
		let mut mem = Memory {
			inner: utils::new_zeroed_box_in(alloc),
		};
		
		mem.reset();
		mem
	}
	
	pub fn reset(&mut self) {
		self.inner[0x5f25] = 6; // default pen color
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
	
	pub fn read<T: Serializable>(&self, addr: u16) -> T {
		T::read(self, addr)
	}
	
	pub fn write<T: Serializable>(&mut self, addr: u16, val: T) {
		val.write(self, addr);
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

impl<'a> MemoryScreen<'a> {
	fn get_addr(x: i16, y: i16) -> Result<(usize, bool), ()> {
		if x < 0 || y < 0 || x >= 128 || y >= 128 { return Err(()) }
		Ok((
			((x / 2) + y * 64) as usize,
			x & 1 == 0,
		))
	}
	
	pub fn get_pixel(&self, x: i16, y: i16) -> Result<u8, ()> {
		let (addr, high) = Self::get_addr(x, y)?;
		Ok(if high {
			self.0[addr] & 0xF
		}else{
			self.0[addr] >> 4
		})
	}
	
	pub fn set_pixel(&mut self, x: i16, y: i16, value: u8) -> Result<(), ()> {
		let (addr, high) = Self::get_addr(x, y)?;
		let old = self.0[addr];
		if high {
			self.0[addr] = (old & 0xF0) | (value & 0xF);
		}else{
			self.0[addr] = (value << 4) | (old & 0xF);
		}
		Ok(())
	}
}

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

pub trait Serializable {
	fn read<A: Allocator>(mem: &Memory<A>, addr: u16) -> Self;
	fn write<A: Allocator>(self, mem: &mut Memory<A>, addr: u16);
}

macro_rules! impl_ser {
	($T:ty $(, $( $rest:tt )*)?) => {
		impl Serializable for $T {
			fn read<A: Allocator>(mem: &Memory<A>, addr: u16) -> Self {
				Self::from_le_bytes(core::array::from_fn(|pos| mem[addr.wrapping_add(pos as u16) as usize]))
			}
			fn write<A: Allocator>(self, mem: &mut Memory<A>, addr: u16) {
				for (pos, byte) in self.to_le_bytes().iter().copied().enumerate() {
					mem[addr.wrapping_add(pos as u16) as usize] = byte;
				}
			}
		}
		
		$( impl_ser!($( $rest )*); )?
	};
	() => {};
}

impl_ser!(u8, i8, u16, i16, u32, i32, P8Num);


pub struct MemoryDrawState<'a, A: Allocator>(&'a mut Memory<A>);

impl<'a, A: Allocator> MemoryDrawState<'a, A> {
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
		[ self.0.read::<i16>(0x5f28), self.0.read::<i16>(0x5f2a) ]
	}
	
	pub fn set_camera_x(&mut self, value: i16) {
		self.0.write(0x5f28, value);
	}
	
	pub fn set_camera_y(&mut self, value: i16) {
		self.0.write(0x5f2a, value);
	}
}


bitflags! {
	#[derive(Copy, Clone)]
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


pub struct MemoryHardwareState<'a, A: Allocator>(&'a mut Memory<A>);

// 0x5f40..0x5f80
impl<'a, A: Allocator> MemoryHardwareState<'a, A> {
	pub fn get_print_defaults(&mut self) -> PrintAttributeFlags {
		PrintAttributeFlags::from_bits_truncate(self.0[0x5f58])
	}
	
	pub fn set_print_defaults(&mut self, flags: PrintAttributeFlags) {
		self.0[0x5f58] = flags.bits();
	}
}
