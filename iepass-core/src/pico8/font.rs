use bitflags::bitflags;

mod generated;

struct Font<'a>(&'a [u8; 0x800]);

impl<'a> Font<'a> {
	const SYSTEM: Font<'static> = Font(&generated::SYSTEM_FONT);
	
	fn width(&self) -> u8 { self.0[0] }
	fn width_ex(&self) -> u8 { self.0[1] }
	fn width_tab(&self) -> u8 { self.0[6] }
	fn width_chr(&self, char: u8) -> u8 {
		let flags = self.flags();
		let adjust = if flags.contains(FontFlags::SIZE_ADUST_EN) && char > 16 {
			let byte = self.0[char as usize / 2];
			let nibble = if char % 2 == 0 { byte & 0x0F } else { byte >> 4 };
			match nibble {
				0..4 => nibble as i8,
				4.. => nibble as i8 - 8,
			}
		} else {
			0
		};
		
		match char {
			0..16 => 0,
			16..128 => self.width().saturating_add_signed(adjust).min(8),
			128..=255 => self.width_ex().saturating_add_signed(adjust).min(8),
		}
	}
	fn height(&self) -> u8 { self.0[2] }
	fn offset(&self) -> (u8, u8) { (self.0[3], self.0[4]) }
	fn flags(&self) -> FontFlags { FontFlags::from_bits_truncate(self.0[5]) }
	
	fn char(&self, char: u8) -> [u8; 8] {
		match char {
			0..16 => [0; 8],
			16.. => self.0.as_chunks().0[char as usize],
		}
	}
}

bitflags! {
	pub struct FontFlags: u8 {
		const SIZE_ADUST_EN = 0b0000_0001;
		const TAB_RELATIVE  = 0b0000_0010;
	}
}