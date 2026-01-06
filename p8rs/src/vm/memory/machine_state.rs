use core::ops::{Deref, DerefMut};
use core::time::Duration;
use bitflags::bitflags;
use p8rs_macros::{p8, TransparentRef};
use thiserror::Error;
use p8rs_types::p8num::P8Num;

use crate::utils::NonZeroNibble;
use super::MemoryAccess;

/// 0x5f00..=0x5f80
pub struct MachineState<'m>(pub(super) &'m mut [u8; 0x80]);

impl MachineState<'_> {
	pub fn reset(&mut self) {
		let rng_state = *self.rnd_state();
		self.fill(0);
		
		*self.rnd_state() = rng_state;
		*self.pen_color() = 6;
		*self.clip_rect() = [0, 0, 128, 128];
		self._set_cursor_y(6);
		
		*self.sprite_addr_map() = SpriteScreenMemoryMap::SPRITE_SHEET;
		*self.screen_addr_map() = SpriteScreenMemoryMap::SCREEN;
		*self.map_addr_map() = 0x20;
		*self.map_width() = 128;
		*self.bitplane_mask() = 0xff;
		
		self.reset_palettes();
	}
	
	pub fn reset_palettes(&mut self) {
		self.reset_palette(Palette::Draw);
		self.reset_palette(Palette::Screen);
		self.reset_palette(Palette::Secondary);
	}
	
	pub fn reset_palette(&mut self, idx: Palette) {
		match idx {
			Palette::Draw      => *self.palette(idx) = [0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f],
			Palette::Screen    => *self.palette(idx) = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f],
			Palette::Secondary => *self.palette(idx) = [0x00, 0x01, 0x12, 0x13, 0x24, 0x15, 0xd6, 0x67, 0x48, 0x49, 0x9a, 0x3b, 0xdc, 0x5d, 0x8e, 0xef],
		}
	}
	
	pub fn palette(&mut self, idx: Palette) -> &mut [u8; 16] {
		let base = idx.base_addr();
		(&mut self[base..base+16]).try_into().unwrap()
	}
	
	/// [x_begin, y_begin, x_end, y_end]
	pub fn clip_rect(&mut self) -> &mut [u8; 4] {
		self.const_slice_mut(0x20)
	}
	
	/// Cursor must be controlled via methods in `Runtime` instead of changing memory directly!
	pub(crate) fn _set_cursor_home_x(&mut self, value: u8) {
		self.write(0x24, value);
	}
	
	pub fn pen_color(&mut self) -> &mut u8 {
		&mut self[0x25]
	}
	
	/// Cursor must be controlled via methods in `Runtime` instead of changing memory directly!
	pub(crate) fn _set_cursor_x(&mut self, val: u8) {
		self.write(0x26, val);
	}
	
	/// Cursor must be controlled via methods in `Runtime` instead of changing memory directly!
	pub(crate) fn _set_cursor_y(&mut self, val: u8) {
		self.write(0x27, val);
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
	
	pub fn screen_transform(&mut self) -> &mut ScreenTransform {
		ScreenTransform::from_bits_mut(&mut self[0x2c])
	}
	
	pub fn devkit_flags(&mut self) -> &mut DevkitFlags {
		DevkitFlags::from_bits_mut(&mut self[0x2d])
	}
	
	pub fn persistence_flags(&mut self) -> &mut PersistenceFlags {
		PersistenceFlags::from_bits_mut(&mut self[0x2e])
	}
	
	pub fn audio_state(&mut self) -> &mut AudioState {
		AudioState::from_bits_mut(&mut self[0x2f])
	}
	
	pub fn pause_state(&mut self) -> &mut PauseState {
		PauseState::from_bits_mut(&mut self[0x30])
	}
	
	pub fn fill_pattern(&mut self) -> &mut FillPatternState {
		FillPatternState::from_bits_mut(self.const_slice_mut(0x31))
	}
	
	pub fn color_flags(&mut self) -> &mut ColorFlags {
		ColorFlags::from_bits_mut(&mut self[0x34])
	}
	
	pub fn line_state(&mut self) -> &mut LineState {
		LineState::from_bits_mut(&mut self[0x35])
	}
	
	pub fn misc_chipset_flags(&mut self) -> &mut MiscChipsetFeatureFlags {
		MiscChipsetFeatureFlags::from_bits_mut(&mut self[0x36])
	}
	
	pub fn editor_state(&mut self) -> &mut EditorState {
		EditorState::from_bits_mut(&mut self[0x37])
	}
	
	pub fn tline_clip_size(&mut self) -> &mut [u8; 2] {
		self.const_slice_mut(0x38)
	}
	
	pub fn tline_clip_offset(&mut self) -> &mut [u8; 2] {
		self.const_slice_mut(0x3a)
	}
	
	pub fn get_line_endpoint(&mut self) -> Option<[i16; 2]> {
		match *self.line_state() {
			LineState::ENDPOINT_UNSET => None,
			_ => Some([ self.read::<i16>(0x3c), self.read::<i16>(0x3e) ]),
		}
	}
	
	pub fn set_line_endpoint(&mut self, value: Option<[i16; 2]>) {
		match value {
			Some([x, y]) => {
				*self.line_state() = LineState::ENDPOINT_SET;
				self.write(0x3c, x);
				self.write(0x3e, y);
			},
			None => {
				*self.line_state() = LineState::ENDPOINT_UNSET;
			},
		}
	}
	
	pub fn audio_effects_flags(&mut self) -> &mut [u8; 4] {
		self.const_slice_mut(0x40)
	}
	
	pub fn rnd_state(&mut self) -> &mut [u8; 8] {
		self.const_slice_mut(0x44)
	}
	
	pub fn btn_state(&mut self) -> &mut [u8; 8] {
		self.const_slice_mut(0x4c)
	}
	
	pub fn sprite_addr_map(&mut self) -> &mut SpriteScreenMemoryMap {
		SpriteScreenMemoryMap::from_bits_mut(&mut self[0x54])
	}
	
	pub fn screen_addr_map(&mut self) -> &mut SpriteScreenMemoryMap {
		SpriteScreenMemoryMap::from_bits_mut(&mut self[0x55])
	}
	
	pub fn map_addr_map(&mut self) -> &mut u8 {
		&mut self[0x56]
	}
	
	pub fn map_width(&mut self) -> &mut u8 {
		&mut self[0x57]
	}
	
	pub fn print_defaults(&mut self) -> &mut PrintDefaults {
		PrintDefaults::from_bits_mut(self.const_slice_mut(0x58))
	}
	
	pub fn btnp_rep_delay(&mut self) -> &mut BtnpRepDelay {
		BtnpRepDelay::from_bits_mut(&mut self[0x5c])
	}
	
	pub fn btnp_rep_interval(&mut self) -> &mut BtnpRepInterval {
		BtnpRepInterval::from_bits_mut(&mut self[0x5d])
	}
	
	pub fn bitplane_mask(&mut self) -> &mut u8 {
		&mut self[0x5e]
	}
	
	pub fn high_color_mode(&mut self) -> &mut u8 {
		&mut self[0x5f]
	}
	
	pub fn high_color_bitfield(&mut self) -> &mut [u8; 16] {
		self.const_slice_mut(0x70)
	}
	
	#[inline(always)]
	pub(crate) fn const_slice_mut<const S: usize>(&mut self, base: u16) -> &mut [u8; S] {
		(&mut self.0[base as usize..base as usize + S]).try_into().unwrap()
	}
	
	pub fn set_pen_color(&mut self, color: P8Num) {
		*self.pen_color() = color.to_integer() as u8;
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct ScreenTransform(u8);

impl ScreenTransform {
	/// Normal mode
	pub const NORMAL: Self = Self(0);
	/// Horizontal stretch, 64x128 screen, left half of normal screen
	pub const STRETCH_HORIZONTAL: Self = Self(1);
	/// Vertical stretch, 128x64 screen, top half of normal screen
	pub const STRETCH_VERTICAL: Self = Self(2);
	/// Both stretch, 64x64 screen, top left quarter of normal screen
	pub const STRETCH_BOTH: Self = Self(3);
	/// Horizontal mirroring, left half copied and flipped to right half
	pub const MIRROR_HORIZONTAL: Self = Self(5);
	/// Vertical mirroring, top half copied and flipped to bottom half
	pub const MIRROR_VERTICAL: Self = Self(6);
	/// Both mirroring, top left quarter copied and flipped to other quarters
	pub const MIRROR_BOTH: Self = Self(7);
	/// Horizontal flip
	pub const FLIP_HORIZONTAL: Self = Self(129);
	/// Vertical flip
	pub const FLIP_VERTICAL: Self = Self(130);
	/// Both flip
	pub const FLIP_BOTH: Self = Self(131);
	/// Clockwise 90 degree rotation
	pub const ROTATE_90: Self = Self(133);
	/// 180 degree rotation (effectively equivalent to 131)
	pub const ROTATE_180: Self = Self(134);
	/// Counterclockwise 90 degree rotation
	pub const ROTATE_270: Self = Self(135);
	
	pub fn new(value: u8) -> Self {
		ScreenTransform(value)
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct DevkitFlags(u8);

bitflags! {
    impl DevkitFlags: u8 {
		/// Enable devkit mode
        const ENABLE        = 1 << 0;
		/// Mouse buttons can be read using btn(4), btn(5), and btn(6)
        const MOUSE_BUTTONS = 1 << 1;
		/// Lock pointer. Movement can be read using stat(38) and stat(39)
        const POINTER_LOCK  = 1 << 2;
		
        const _ = !0;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct PersistenceFlags(u8);

bitflags! {
    impl PersistenceFlags: u8 {
		/// Persist current palette scheme at 0x5f00..=0x5f1f.
        const PALETTE       = 1 << 0;
		/// Persist high-color mode configuration at 0x5f5f..=0x5f7f.
        const HIGH_COLOR    = 1 << 1;
		/// Persist audio effect switches at 0x5f40..=0x5f43.
        const AUDIO_EFFECTS = 1 << 2;
		/// Persist read/write masks at 0x5f5e.
        const RW_MASK       = 1 << 3;
		/// Persist default print attributes at 0x5f58..=0x5f5b.
        const PRINT_ATTRS   = 1 << 4;
		/// Persist fill pattern information at 0x5f31..=0x5f33.
        const FILL_PATTERN  = 1 << 5;
		
        const _ = !0;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct AudioState(u8);

impl AudioState {
	/// Audio engine runs in game and pauses in menu.
	pub const NORMAL: Self = Self(0);
	/// Audio engine is always paused.
	pub const PAUSED: Self = Self(1);
	/// Audio engine is always running.
	pub const ALWAYS: Self = Self(2);
	
	pub fn new(value: u8) -> Self {
		AudioState(value)
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct PauseState(u8);

impl PauseState {
	/// Pause menu is enabled.
	pub const NORMAL: Self = Self(0);
	/// Suppress the next attempt to bring up the pause menu.
	pub const SUPPRESS: Self = Self(1);
	
	pub fn new(value: u8) -> Self {
		PauseState(value)
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct FillPatternState([u8; 3]);

impl FillPatternState {
	pub fn new(pattern: u16, flags: FillPatternFlags) -> Self {
		let [lo, hi] = u16::to_le_bytes(pattern);
		FillPatternState([lo, hi, flags.bits()])
	}
	
	pub fn flags(&mut self) -> &mut FillPatternFlags {
		FillPatternFlags::from_bits_mut(&mut self.0[2])
	}
	
	pub fn pattern(&self) -> u16 {
		u16::from_le_bytes([self.0[0], self.0[1]])
	}
	
	pub fn expand(&self) -> Option<[[bool; 4]; 4]> {
		let lo = self.0[0];
		let hi = self.0[1];
		
		if lo == 0 && hi == 0 {
			None
		} else {
			Some([
				[hi & 1 << 7 != 0, hi & 1 << 6 != 0, hi & 1 << 5 != 0, hi & 1 << 4 != 0],
				[hi & 1 << 3 != 0, hi & 1 << 2 != 0, hi & 1 << 1 != 0, hi & 1 << 0 != 0],
				[lo & 1 << 7 != 0, lo & 1 << 6 != 0, lo & 1 << 5 != 0, lo & 1 << 4 != 0],
				[lo & 1 << 3 != 0, lo & 1 << 2 != 0, lo & 1 << 1 != 0, lo & 1 << 0 != 0],
			])
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct FillPatternFlags(u8);

bitflags! {
    impl FillPatternFlags: u8 {
		/// Enables transparency
        const TRANSPARENT = 1 << 0;
		/// When drawing sprites, fill pattern will determine nibble of secondary palette to use
        const REMAP_SPRITES = 1 << 1;
		/// Other drawing functions that accept fill pattern will use it to determine nibble of secondary palette to use
        const REMAP_OTHER = 1 << 2;
		
        const _ = !0;
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct ColorFlags(u8);

bitflags! {
    impl ColorFlags: u8 {
		/// All API functions that take color also accept pattern information.
        const INCLUDE_PATTERN = 1 << 0;
		/// Invert flag in color with pattern arguments will be validated.
        const VALIDATE_INVERT = 1 << 1;
		
        const _ = !0;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct LineState(u8);

impl LineState {
	/// Line endpoint is set as present at 0x5f3c..=0x5f3f
	pub const ENDPOINT_SET: Self = Self(0);
	/// Line endpoint is not set.
	pub const ENDPOINT_UNSET: Self = Self(1);
	
	pub fn new(value: u8) -> Self {
		LineState(value)
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct MiscChipsetFeatureFlags(u8);

bitflags! {
	impl MiscChipsetFeatureFlags: u8 {
		/// The undocumented multiscreen feature is enabled
        const MULTI_SCREEN       = 1 << 0;
		/// The diameter of circles drawn using circ() and circfill() will be increased by 1 pixel rightward and 1 pixel downward if the fractional part of the radius is .5 or greater
        const EVEN_RADIUS_CIRC       = 1 << 1;
		/// Automatic newlines are no longer added after each call to print()
        const NO_PRINT_NEWLINE   = 1 << 2;
		/// Causes sprite 0 in map() and tline() to be rendered as opaque (like other sprites) instead of the usual transparent
        const OPAQUE_ZERO_SPRITE = 1 << 3;
		/// 0x5f59..0x5f5b will be interpreted as default values for sget, mget, and pget
        const PIXEL_DEFAULTS     = 1 << 4;
		/// The dampen filter used for the undocumented PCM audio channel (serial(0x808,...)) is disabled
        const NO_PCM_DAMPEN      = 1 << 5;
		/// Automatic screen scrolling for print() without coordinate parameters is disabled
        const NO_PRINT_SCROLL    = 1 << 6;
		/// Automatic character wrap for print() is enabled
        const PRINT_WRAP      = 1 << 7;
		
        const _ = !0;
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct EditorState(u8);

impl EditorState {
	/// ROM will be copied to RAM whenever user exits editor mode. 0x0000..=0x42ff memory range will be overwritten.
	pub const NORMAL: Self = Self(0);
	/// ROM will *not* be copied to RAM whenever user exits editor mode. RAM memory will be preserved.
	pub const PRESERVE_RAM: Self = Self(1);
	
	pub fn new(value: u8) -> Self {
		EditorState(value)
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct SpriteScreenMemoryMap(u8);

impl SpriteScreenMemoryMap {
	/// Sprite sheet (0x0000..=0x0fff)
	pub const SPRITE_SHEET: Self = Self(0x00);
	/// Screen buffer (0x6000..=0x6fff)
	pub const SCREEN: Self = Self(0x60);
	/// Extended RAM buffer 0 (0x8000..=0x8fff)
	pub const EXT_RAM_0: Self = Self(0x80);
	/// Extended RAM buffer 1 (0xa000..=0xafff)
	pub const EXT_RAM_1: Self = Self(0xa0);
	/// Extended RAM buffer 2 (0xc000..=0xcfff)
	pub const EXT_RAM_2: Self = Self(0xc0);
	/// Extended RAM buffer 3 (0xe000..=0xefff)
	pub const EXT_RAM_3: Self = Self(0xe0);
	
	pub fn new(value: u8) -> Self {
		SpriteScreenMemoryMap(value)
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct PrintDefaults([u8; 4]);

impl PrintDefaults {
	pub fn new(flags: PrintDefaultsFlags,
	           char_w: Option<NonZeroNibble>, char_h: Option<NonZeroNibble>,
	           char_w2: Option<NonZeroNibble>, tab_w: Option<NonZeroNibble>,
	           offset_x: Option<NonZeroNibble>, offset_y: Option<NonZeroNibble>,
	) -> Self {
		PrintDefaults([
			flags.bits(),
			char_w.map_or(0, |nib| nib.get()) | (char_h.map_or(0, |nib| nib.get()) << 4),
			char_w2.map_or(0, |nib| nib.get()) | (tab_w.map_or(0, |nib| nib.get()) << 4),
			offset_x.map_or(0, |nib| nib.get()) | (offset_y.map_or(0, |nib| nib.get()) << 4),
		])
	}
	
	pub fn flags(&mut self) -> &mut PrintDefaultsFlags {
		PrintDefaultsFlags::from_bits_mut(&mut self.0[0])
	}
	
	pub fn get_char_w(&mut self) -> Option<NonZeroNibble> {
		NonZeroNibble::new(self.0[1] & 0x0F)
	}
	
	pub fn set_char_w(&mut self, val: Option<NonZeroNibble>) {
		self.0[1] = (self.0[1] & 0xF0) | val.map_or(0, |nib| nib.get());
	}
	
	pub fn get_char_h(&mut self) -> Option<NonZeroNibble> {
		NonZeroNibble::new(self.0[1] >> 4)
	}
	
	pub fn set_char_h(&mut self, val: Option<NonZeroNibble>) {
		self.0[1] = (self.0[1] & 0x0F) | (val.map_or(0, |nib| nib.get()) << 4);
	}
	
	pub fn get_char_w2(&mut self) -> Option<NonZeroNibble> {
		NonZeroNibble::new(self.0[2] & 0x0F)
	}
	
	pub fn set_char_w2(&mut self, val: Option<NonZeroNibble>) {
		self.0[2] = (self.0[2] & 0xF0) | val.map_or(0, |nib| nib.get());
	}
	
	pub fn get_tab_w(&mut self) -> Option<NonZeroNibble> {
		NonZeroNibble::new(self.0[2] >> 4)
	}
	
	pub fn set_tab_w(&mut self, val: Option<NonZeroNibble>) {
		self.0[2] = (self.0[2] & 0x0F) | (val.map_or(0, |nib| nib.get()) << 4);
	}
	
	pub fn get_offset_x(&mut self) -> Option<NonZeroNibble> {
		NonZeroNibble::new(self.0[3] & 0x0F)
	}
	
	pub fn set_offset_x(&mut self, val: Option<NonZeroNibble>) {
		self.0[3] = (self.0[3] & 0xF0) | val.map_or(0, |nib| nib.get());
	}
	
	pub fn get_offset_y(&mut self) -> Option<NonZeroNibble> {
		NonZeroNibble::new(self.0[3] >> 4)
	}
	
	pub fn set_offset_y(&mut self, val: Option<NonZeroNibble>) {
		self.0[3] = (self.0[3] & 0x0F) | (val.map_or(0, |nib| nib.get()) << 4);
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct PrintDefaultsFlags(u8);

bitflags! {
    impl PrintDefaultsFlags: u8 {
        const ENABLE        = 1 << 0;
        const PADDING       = 1 << 1;
        const WIDE          = 1 << 2;
        const TALL          = 1 << 3;
        const SOLID_BG      = 1 << 4;
        const INVERT        = 1 << 5;
        const DOTTY         = 1 << 6;
        const CUSTOM_FONT   = 1 << 7;
		
        const _ = !0;
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct BtnpRepDelay(u8);

impl BtnpRepDelay {
	/// Use system default button repeat delay
	pub const DEFAULT: Self = Self(0);
	/// Disable button repeat
	pub const DISABLED: Self = Self(255);
	
	pub fn new(value: u8) -> Self {
		BtnpRepDelay(value)
	}
	
	pub fn as_duration(&self) -> Option<Duration> {
		if *self == Self::DISABLED || *self == Self::DEFAULT {
			None
		} else {
			Some(Duration::from_secs_f32(self.0 as f32 / 30.0))
		}
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

#[derive(Copy, Clone, Debug, Error)]
#[error("Cannot convert duration to this type.")]
pub struct TryFromDurationError;

impl TryFrom<Duration> for BtnpRepDelay {
	type Error = TryFromDurationError;
	
	fn try_from(value: Duration) -> Result<Self, Self::Error> {
		let steps = (P8Num::new(value.as_secs_f32()) * p8!(30)).round().to_integer();
		if steps >= 1 && steps <= 254 {
			Ok(BtnpRepDelay(steps as u8))
		} else {
			Err(TryFromDurationError)
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TransparentRef)]
#[repr(transparent)]
pub struct BtnpRepInterval(u8);

impl BtnpRepInterval {
	/// Use system default button repeat interval
	pub const DEFAULT: Self = Self(0);
	
	pub fn new(value: u8) -> Self {
		BtnpRepInterval(value)
	}
	
	pub fn as_duration(&self) -> Option<Duration> {
		if *self == Self::DEFAULT {
			None
		} else {
			Some(Duration::from_secs_f32(self.0 as f32 / 30.0))
		}
	}
	
	pub fn get(self) -> u8 {
		self.0
	}
}

impl TryFrom<Duration> for BtnpRepInterval {
	type Error = TryFromDurationError;
	
	fn try_from(value: Duration) -> Result<Self, Self::Error> {
		let steps = (P8Num::new(value.as_secs_f32()) * p8!(30)).round().to_integer();
		if steps >= 1 && steps <= 255 {
			Ok(BtnpRepInterval(steps as u8))
		} else {
			Err(TryFromDurationError)
		}
	}
}
