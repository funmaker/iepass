use std::ops::{Deref, DerefMut};
use bitflags::bitflags;

use super::MemoryAccess;

/// 0x5f00..=0x5f80
pub struct MachineState<'a>(pub(super) &'a mut [u8; 0x80]);

impl MachineState<'_> {
	pub fn reset(&mut self) {
		*self.pen_color() = 6;
		*self.clip_rect() = [0, 0, 128, 128];
		*self.cursor_position() = [0, 6];
		*self.cursor_home_x() = 0;
		*self.sprite_addr_map() = 0x00;
		*self.screen_addr_map() = 0x60;
		*self.map_addr_map() = 0x20;
		*self.map_width() = 128;
		
		self.reset_palette();
	}
	
	pub fn reset_palette(&mut self) {
		*self.palette(Palette::Draw)      = [0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
		*self.palette(Palette::Screen)    = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
		*self.palette(Palette::Secondary) = [0x00, 0x01, 0x12, 0x13, 0x24, 0x15, 0xd6, 0x67, 0x48, 0x49, 0x9a, 0x3b, 0xdc, 0x5d, 0x8e, 0xef];
	}
	
	pub fn palette(&mut self, idx: Palette) -> &mut [u8; 16] {
		let base = idx.base_addr();
		(&mut self[base..base+16]).try_into().unwrap()
	}
	
	pub fn cursor_home_x(&mut self) -> &mut u8 {
		&mut self[0x24]
	}
	
	pub fn pen_color(&mut self) -> &mut u8 {
		&mut self[0x25]
	}
	
	pub fn cursor_position(&mut self) -> &mut [u8; 2] {
		(&mut self[0x26..=0x27]).try_into().unwrap()
	}
	
	pub fn get_camera_position(&self) -> [i16; 2] {
		[ self.read::<i16>(0x28), self.read::<i16>(0x2a) ]
	}
	
	pub fn set_camera_x(&mut self, value: i16) {
		self.write(0x28, value);
	}
	
	pub fn set_camera_y(&mut self, value: i16) {
		self.write(0x2a, value);
	}
	
	/// [x_begin, y_begin, x_end, y_end]
	pub fn clip_rect(&mut self) -> &mut [u8; 4] {
		(&mut self[0x20..=0x23]).try_into().unwrap()
	}
	
	pub fn get_misc_chipset_flags(&self) -> MiscChipsetFeatureFlags {
		MiscChipsetFeatureFlags::from_bits_truncate(self[0x36])
	}
	
	pub fn set_misc_chipset_flags(&mut self, flags: MiscChipsetFeatureFlags) {
		self[0x36] = flags.bits()
	}
	
	pub fn get_print_defaults(&self) -> PrintAttributeFlags {
		PrintAttributeFlags::from_bits_truncate(self[0x58])
	}
	
	pub fn set_print_defaults(&mut self, flags: PrintAttributeFlags) {
		self[0x58] = flags.bits();
	}
	
	pub fn sprite_addr_map(&mut self) -> &mut u8 {
		&mut self[0x54]
	}
	
	pub fn screen_addr_map(&mut self) -> &mut u8 {
		&mut self[0x55]
	}
	
	pub fn map_addr_map(&mut self) -> &mut u8 {
		&mut self[0x56]
	}
	
	pub fn map_width(&mut self) -> &mut u8 {
		&mut self[0x57]
	}
}

impl Deref for MachineState<'_> {
	type Target = [u8; 0x80];
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for MachineState<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Palette {
	Draw = 0,
	Screen = 1,
	Secondary = 2,
}

impl Palette {
	pub fn new(idx: impl TryInto<u8>) -> Option<Self> {
		let idx = idx.try_into().ok()?;
		match idx {
			0 => Some(Self::Draw),
			1 => Some(Self::Screen),
			2 => Some(Self::Secondary),
			_ => None,
		}
	}
	
	fn base_addr(self) -> usize {
		match self {
			Palette::Draw => 0x00,
			Palette::Screen => 0x10,
			Palette::Secondary => 0x60,
		}
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
	
	#[derive(Copy, Clone)]
	pub struct MiscChipsetFeatureFlags: u8 {
		/// the undocumented multi-screen feature is enabled
        const MULTI_SCREEN       = 1 << 0;
		/// the diameter of circles drawn using circ() and circfill() will be increased by 1 pixel rightward and 1 pixel downward if the fractional part of the radius is .5 or greater
        const FRACT_CIRCLE       = 1 << 1;
		/// automatic newlines are no longer added after each call to print()
        const NO_PRINT_NEWLINE   = 1 << 2;
		/// causes sprite 0 in map() and tline() to be rendered as opaque (like other sprites) instead of the usual transparent
        const OPAQUE_ZERO_SPRITE = 1 << 3;
		/// 0x5f59..0x5f5b will be interpreted as default values for sget, mget, and pget
        const PIXEL_DEFAULTS     = 1 << 4;
		/// the dampen filter used for the undocumented PCM audio channel (serial(0x808,...)) is disabled
        const NO_PCM_DAMPEN      = 1 << 5;
		/// automatic screen scrolling for print() without coordinate parameters is disabled
        const NO_PRINT_SCROLL    = 1 << 6;
		/// automatic character wrap for print() is enabled
        const PRINT_WRAP         = 1 << 7;
	}
}
