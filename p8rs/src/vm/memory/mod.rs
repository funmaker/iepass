use core::ops::{Deref, DerefMut};
use core::fmt::{Debug, Formatter};
use bytemuck::Zeroable;
use p8rs_types::p8num::P8Num;

pub mod sprites;
pub mod map;
pub mod sprite_flags;
pub mod music;
pub mod sound_effects;
pub mod machine_state;
pub mod screen;
pub mod painter;

use sprites::Sprites;
use map::Map;
use sprite_flags::SpriteFlags;
use music::Music;
use sound_effects::SoundEffects;
use machine_state::MachineState;
use screen::Screen;
use painter::Painter;

#[derive(Clone, Zeroable)]
#[repr(transparent)]
pub struct Memory([u8; 0x10000]);

impl Memory {
	pub fn reset(&mut self) {
		self.machine_state().reset();
	}
	
	pub fn sprites(&mut self) -> Sprites<'_> {
		// TODO: Handle map conflicts
		let base_addr = match self.machine_state().sprite_addr_map().get() {
			0x00 => 0x0000,
			0x60 => 0x6000,
			0x80 => 0x8000,
			0xa0 => 0xa000,
			0xc0 => 0xc000,
			0xe0 => 0xe000,
			_    => 0x0000,
		};
		
		Sprites(self.const_slice(base_addr))
	}
	
	pub fn map(&mut self) -> Map<'_> {
		// TODO: Validate
		let base_addr = match *self.machine_state().map_addr_map() {
			0x10..=0x1f => 0x3000,
			0x20..=0x2f => 0x2000,
			0x30..=0x3f => 0x3000,
			0x80..=0xff => 0x8000,
			_           => 0x0000,
		};
		
		let width = match *self.machine_state().map_width() {
			0 => 256,
			n => n as usize,
		};
		
		Map {
			memory: self.const_slice(base_addr),
			width,
			height: 64,
		}
	}
	
	pub fn sprite_flags(&mut self) -> SpriteFlags<'_> {
		SpriteFlags(self.const_slice(0x3000))
	}
	
	pub fn music(&mut self) -> Music<'_> {
		Music(self.const_slice(0x3100))
	}
	
	pub fn sound_effects(&mut self) -> SoundEffects<'_> {
		SoundEffects(self.const_slice(0x3200))
	}
	
	pub fn persistent_data(&mut self) -> &mut [u8; 256] {
		self.const_slice(0x5e00)
	}
	
	pub fn machine_state(&mut self) -> MachineState<'_> {
		MachineState(self.const_slice(0x5f00))
	}
	
	pub fn gpio(&mut self) -> &mut [u8; 128] {
		self.const_slice(0x5f80)
	}
	
	pub fn screen(&mut self) -> Screen<'_> {
		// TODO: Handle map conflicts
		let base_addr = match self.machine_state().screen_addr_map().get() {
			0x00 => 0x0000,
			0x60 => 0x6000,
			0x80 => 0x8000,
			0xa0 => 0xa000,
			0xc0 => 0xc000,
			0xe0 => 0xe000,
			_    => 0x0000,
		};
		
		Screen(self.const_slice(base_addr))
	}
	
	pub fn painter(&mut self) -> Painter<'_> {
		Painter::new(self)
	}
	
	#[inline(always)]
	pub(crate) fn const_slice<const S: usize>(&mut self, base: u16) -> &mut [u8; S] {
		(&mut self.0[base as usize..base as usize + S]).try_into().unwrap()
	}
}

impl Deref for Memory {
	type Target = [u8; 0x10000];
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Memory {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Debug for Memory {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str("Memory[64KB]")
	}
}

#[cfg(feature = "defmt")]
impl defmt::Format for Memory {
	fn format(&self, fmt: defmt::Formatter) {
		defmt::write!(fmt, "Memory[64KB]");
	}
}

pub trait Serializable {
	fn read<const S: usize>(mem: &[u8; S], addr: u16) -> Self;
	fn write<const S: usize>(self, mem: &mut [u8; S], addr: u16);
}

macro_rules! impl_ser {
	($T:ty $(, $( $rest:tt )*)?) => {
		impl Serializable for $T {
			#[inline(always)]
			fn read<const S: usize>(mem: &[u8; S], addr: u16) -> Self {
				Self::from_le_bytes(core::array::from_fn(|pos| mem[addr.wrapping_add(pos as u16) as usize % S]))
			}
			
			#[inline(always)]
			fn write<const S: usize>(self, mem: &mut [u8; S], addr: u16) {
				for (pos, byte) in self.to_le_bytes().iter().copied().enumerate() {
					mem[addr.wrapping_add(pos as u16) as usize % S] = byte;
				}
			}
		}
		
		$( impl_ser!($( $rest )*); )?
	};
	() => {};
}

impl_ser!(u8, i8, u16, i16, u32, i32, P8Num);

pub trait MemoryAccess {
	fn read<T: Serializable>(&self, addr: u16) -> T;
	fn write<T: Serializable>(&mut self, addr: u16, val: T);
}

impl<const S: usize> MemoryAccess for [u8; S] {
	fn read<T: Serializable>(&self, addr: u16) -> T {
		T::read(self, addr)
	}
	
	fn write<T: Serializable>(&mut self, addr: u16, val: T) {
		val.write(self, addr);
	}
}
