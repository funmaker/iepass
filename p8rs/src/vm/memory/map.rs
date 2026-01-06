use super::Memory;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MapOffset {
	Upper(u16),    // Starts at 0x3000..0x4000
	Lower(u16),    // Starts at 0x2000..0x3000
	Extended(u16), // Starts at 0x8000..
}

/// Usually 0x2000..=0x3fff
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Map {
	offset: MapOffset,
	width: u16,
	height: u16,
}

impl Map {
	pub(super) fn new(offset: u16, memory: &mut Memory) -> Map {
		let offset = match offset {
			0x2000..0x3000 => MapOffset::Lower(offset - 0x2000),
			0x3000..0x4000 => MapOffset::Upper(offset - 0x3000),
			0x8000.. => MapOffset::Extended(offset - 0x8000),
			_ => panic!("Invalid map offset {offset}. Should be 0x2000..0x4000 or 0x8000.."),
		};
		
		let width = match *memory.machine_state().map_width() {
			0 => 256,
			n => n as u16,
		};
		
		let height = match offset {
			MapOffset::Upper(offset) => (0x2000 - offset) / width,
			MapOffset::Lower(offset) => (0x1000 - offset) / width,
			MapOffset::Extended(offset) => (0x8000 - offset) / width,
		};
		
		Map { offset, width, height }
	}
}
