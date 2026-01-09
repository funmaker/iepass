use super::Memory;
use super::screen::Screen;
use super::sprites::Sprites;
use super::map::Map;
use super::painter::Painter;

pub struct Graphics {
	screen: u16,
	sprites: u16,
	map: u16,
}

impl Graphics {
	pub(super) fn new(memory: &mut Memory) -> Graphics {
		let mut ms = memory.machine_state();
		
		let mut sprites = match ms.sprite_addr_map().get() {
			0x00 => 0x0000,
			0x60 => 0x6000,
			0x80 => 0x8000,
			0xa0 => 0xa000,
			0xc0 => 0xc000,
			0xe0 => 0xe000,
			_    => 0x0000,
		};
		
		let mut screen = match ms.screen_addr_map().get() {
			0x00 => 0x0000,
			0x60 => 0x6000,
			0x80 => 0x8000,
			0xa0 => 0xa000,
			0xc0 => 0xc000,
			0xe0 => 0xe000,
			_    => 0x6000,
		};
		
		let map = match *ms.map_addr_map() {
			base @ 0x10..=0x1f => 0x1000 + (base & 0x0f) as u16 * 0x100,
			base @ 0x20..=0x2f => 0x2000 + (base & 0x0f) as u16 * 0x100,
			base @ 0x30..=0x3f => 0x1000 + (base & 0x0f) as u16 * 0x100,
			base @ 0x80..=0xff => 0x8000 + (base & 0x7f) as u16 * 0x100,
			_                      => 0x2000,
		};
		
		if map >= 0x8000 {
			if sprites >= map & 0xE000 {
				sprites = 0x0000;
			}
			if screen >= map & 0xE000 {
				screen = 0x6000;
			}
		}
		
		Graphics {
			sprites,
			screen,
			map,
		}
	}
	
	pub fn screen(&self) -> Screen {
		Screen::new(self.screen)
	}
	
	pub fn sprites(&self) -> Sprites {
		Sprites::new(self.sprites)
	}
	
	pub fn map(&self, memory: &mut Memory) -> Map {
		Map::new(self.map, memory)
	}
	
	pub fn painter(&self, memory: &mut Memory) -> Painter {
		Painter::new(self.screen(), memory)
	}
}
