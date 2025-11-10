use core::alloc::Allocator;
use core::ops::{Deref, DerefMut};
use core::array;
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
		self.draw_state().reset();
		self.hardware_state().reset();
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

/// 0x6000..0x7fff
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

/// 0x5f00..0x5f40
pub struct MemoryDrawState<'a, A: Allocator> (&'a mut Memory<A>);

impl<'a, A: Allocator> MemoryDrawState<'a, A> {
	pub fn reset(&mut self) {
		*self.pen_color() = 6;
		*self.clip_rect() = [0, 0, 128, 128];
		*self.cursor_position() = [0, 6];
		*self.cursor_home_x() = 0;
		
		self.reset_palette();
	}
	
	pub fn reset_palette(&mut self) {
		*self.palette(Palette::Draw) = array::from_fn(|i| i as u8);
		*self.palette(Palette::Screen) = array::from_fn(|i| i as u8);
		
		self.palette(Palette::Draw)[0] = 0x10; // transparent
	}
	
	pub fn palette(&mut self, idx: Palette) -> &mut [u8; 16] {
		let base = idx.base_addr() as usize;
		(&mut self.0[base..base+16]).try_into().unwrap()
	}
	
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

/// 0x5f40..0x5f80
pub struct MemoryHardwareState<'a, A: Allocator>(&'a mut Memory<A>);

impl<'a, A: Allocator> MemoryHardwareState<'a, A> {
	pub fn reset(&mut self) {
		self.0[0x5F55] = 0x60; // default screen mapping
		self.0[0x5F56] = 0x20; // default map mapping
		self.0[0x5F57] = 128; // default map size
	}
	
	pub fn get_print_defaults(&mut self) -> PrintAttributeFlags {
		PrintAttributeFlags::from_bits_truncate(self.0[0x5f58])
	}
	
	pub fn set_print_defaults(&mut self, flags: PrintAttributeFlags) {
		self.0[0x5f58] = flags.bits();
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Palette {
	Draw = 0,
	Screen = 1,
	SecondaryScreen = 2,
}

impl Palette {
	pub fn new(idx: impl TryInto<u8>) -> Option<Self> {
		let idx = idx.try_into().ok()?;
		match idx {
			0 => Some(Self::Draw),
			1 => Some(Self::Screen),
			2 => Some(Self::SecondaryScreen),
			_ => None,
		}
	}
	
	pub fn base_addr(self) -> u16 {
		match self {
			Palette::Draw => 0x5f00,
			Palette::Screen => 0x5f10,
			Palette::SecondaryScreen => 0x5f60,
		}
	}
}
