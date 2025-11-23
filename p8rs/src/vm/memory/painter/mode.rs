use crate::vm::memory::machine_state::{FillPatternFlags, Palette};
use crate::vm::memory::Memory;

pub trait PainterMode {
	fn new(memory: &mut Memory) -> Self;
	
	fn fg(&mut self, color: Option<u8>) -> Option<u8>;
	fn bg(&mut self, color: Option<u8>) -> Option<u8>;
}

pub struct PenMode {
	fg: u8,
	bg: Option<u8>,
}

impl PainterMode for PenMode {
	fn new(memory: &mut Memory) -> Self {
		let pen_color = *memory.machine_state().pen_color();
		let fill_flags = *memory.machine_state().fill_pattern().flags();
		let pal = *memory.machine_state().palette(Palette::Draw);
		let bg_opque = !fill_flags.contains(FillPatternFlags::TRANSPARENT);
		
		if fill_flags.contains(FillPatternFlags::REMAP_OTHER) {
			let pal2 = *memory.machine_state().palette(Palette::Secondary);
			
			let map_col = pal[(pen_color & 0xF) as usize] & 0x0F;
			let color = pal2[map_col as usize];
			let fg = color & 0x0F;
			let bg = bg_opque.then_some(color >> 4);
			
			Self { fg, bg }
		} else {
			let pal = *memory.machine_state().palette(Palette::Draw);
			let fg = pal[(pen_color & 0xF) as usize] & 0x0F;
			let bg = bg_opque.then_some(pal[(pen_color >> 4) as usize] & 0x0F);
			
			Self { fg, bg }
		}
	}
	
	fn fg(&mut self, _color: Option<u8>) -> Option<u8> {
		Some(self.fg)
	}
	
	fn bg(&mut self, _color: Option<u8>) -> Option<u8> {
		self.bg
	}
}

pub struct SpriteMode {
	fg: [Option<u8>; 16],
	bg: [Option<u8>; 16],
}

impl PainterMode for SpriteMode {
	fn new(memory: &mut Memory) -> Self {
		let fill_flags = *memory.machine_state().fill_pattern().flags();
		let pal = *memory.machine_state().palette(Palette::Draw);
		
		let mut fg = pal.map(|col| (col < 16).then_some(col));
		let mut bg = fg;
		
		if fill_flags.contains(FillPatternFlags::REMAP_SPRITES) {
			let pal2 = *memory.machine_state().palette(Palette::Secondary);
			
			fg = fg.map(|col| col.map(|col| pal2[col as usize] & 0x0F));
			
			if fill_flags.contains(FillPatternFlags::TRANSPARENT) {
				bg = [None; 16];
			} else {
				bg = bg.map(|col| col.map(|col| pal2[col as usize] >> 4));
			};
		}
		
		Self { fg, bg }
	}
	
	fn fg(&mut self, color: Option<u8>) -> Option<u8> {
		color.and_then(|col| self.fg[(col & 0x0F) as usize])
	}
	
	fn bg(&mut self, color: Option<u8>) -> Option<u8> {
		color.and_then(|col| self.bg[(col & 0x0F) as usize])
	}
}
