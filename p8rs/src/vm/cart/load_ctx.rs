use alloc::borrow::Cow;
use core::alloc::Allocator;
use thiserror::Error;
use p8rs_piccolo::ExternError;
use p8rs_types::p8scii;
use crate::vm::memory::sprites::Sprites;
use crate::vm::P8rs;

pub struct CartLoadContext<'vm, 'c, A: Allocator + 'static> {
	pub vm: &'vm mut P8rs<A>,
	pub lua_code: Cow<'c, [u8]>,
	pub gfx_loaded: bool,
	pub gff_loaded: bool,
	pub map_loaded: bool,
}

impl<'vm, 'c, A: Allocator + 'static> CartLoadContext<'vm, 'c, A> {
	pub fn new(vm: &'vm mut P8rs<A>) -> Self {
		CartLoadContext {
			vm,
			lua_code: Cow::Borrowed(b""),
			gfx_loaded: false,
			gff_loaded: false,
			map_loaded: false,
		}
	}
	
	pub fn load_section(&mut self, name: &[u8], body: &'c [u8]) -> Result<(), CartLoadError> {
		debug!("load_section: Loading cartridge section {} (len: {})", p8scii::Display(name), body.len());
		
		match name {
			b"__lua__" => self.load_lua_section(body)?,
			b"__gfx__" => self.load_gfx_section(body)?,
			b"__gff__" => self.load_gff_section(body)?,
			b"__map__" => self.load_map_section(body)?,
			_ => { warn!("load_section: Unknown section name {}", p8scii::Display(name)); }
		}
		
		Ok(())
	}
	
	fn load_lua_section(&mut self, data: &'c [u8]) -> Result<(), CartLoadError> {
		if self.lua_code.is_empty() {
			self.lua_code = Cow::Borrowed(data);
		} else {
			self.lua_code.to_mut().extend_from_slice(data);
		}
		Ok(())
	}
	
	fn load_gfx_section(&mut self, data: &[u8]) -> Result<(), CartLoadError> {
		let memory = self.vm.memory();
		let sprites = memory.sprites();
		
		if self.gfx_loaded {
			debug!("load_gfx_section: GFX section was already loaded, overwriting data.");
		}
		
		// todo: figure out pico8 behaviour, if load_ctx.long_map_loaded { 64 } else { 128 }
		let mut written = 0;
		for res in hex_iter::<1>(data) {
			if res.col > Sprites::WIDTH as usize || res.row > Sprites::HEIGHT as usize { continue }
			
			sprites.set_pixel(memory, res.col as u8, res.row as u8, res.value);
			written += 1;
		}
		
		self.gfx_loaded = true;
		
		if written <= 0 {
			debug!("load_gfx_section: GFX section loaded: empty section!");
		} else {
			debug!("load_gfx_section: GFX section loaded: 0x{:X} bytes written to memory", written);
		}
		
		Ok(())
	}
	
	fn load_gff_section(&mut self, data: &[u8]) -> Result<(), CartLoadError> {
		let mut flags = self.vm.memory().sprite_flags();
		
		if self.gff_loaded {
			debug!("load_gff_section: GFF section was already loaded, overwriting data.")
		}
		
		let mut written = 0;
		for res in hex_iter::<2>(data) {
			if res.col > 128 || res.row > 2 { continue }
			
			flags[(res.col + res.row * 128) as u8] = res.value;
			written += 1;
		}
		
		if written == -1 {
			debug!("load_gff_section: GFF section loaded: empty section!");
		} else {
			debug!("load_gff_section: GFF section loaded: 0x{:X} bytes written to memory", written);
		}
		
		Ok(())
	}
	
	fn load_map_section(&mut self, data: &[u8]) -> Result<(), CartLoadError> {
		let memory = self.vm.memory();
		let map = memory.map();
		
		if self.map_loaded {
			debug!("load_map_section: MAP section was already loaded, overwriting data.")
		}
		
		// todo: figure out pico8 behaviour, if load_ctx.long_map_loaded { 64 } else { 128 }
		let mut written = 0;
		for res in hex_iter::<2>(data) {
			if res.col > map.width() as usize || res.row > map.height() as usize { continue }
			
			map.set_sprite(memory, res.col as u16, res.row as u16, res.value);
			written += 1;
		}
		
		if written == -1 {
			debug!("load_map_section: MAP section loaded: empty section!");
		} else {
			debug!("load_map_section: MAP section loaded: 0x{:X} bytes written to memory", written);
		}
		
		Ok(())
	}
}

struct HexIterValue {
	value: u8,
	row: usize,
	col: usize,
}

fn hex_iter<const WORD: usize>(lines: &[u8]) -> impl Iterator<Item=HexIterValue> {
	lines.split(|&b| b == b'\n' || b == b'\r')
	     .filter(|line| !line.is_empty())
	     .enumerate()
	     .flat_map(|(row, line)|
		     line.as_chunks::<WORD>().0
		         .iter()
		         .enumerate()
		         .map(move |(col, chunk)| HexIterValue {
			         row,
			         col,
			         value: chunk.iter()
			                     .copied()
			                     .map(hex_value)
			                     .fold(0, |acc, val| (acc << 4) + val.unwrap_or(0)),
		         }))
}

fn hex_value(char: u8) -> Option<u8> {
	match char {
		b'0'..=b'9' => Some(char - b'0'),
		b'a'..=b'f' => Some(char - b'a' + 10),
		b'A'..=b'F' => Some(char - b'A' + 10),
		_ => None,
	}
}

#[derive(Error, Debug)]
pub enum CartLoadError {
	#[error("Could not load cartridge, invalid input data.")]
	InvalidInputData,
	#[error("Invalid cartridge header, expected 'pico-8 cartridge'")]
	InvalidHeader,
	#[error("Could not load cartridge, no data sections detected.")]
	NoDataSection,
	#[error("Could not load cartridge, lua code contains invalid UTF-8 characters.")]
	InvalidLuaUnicode,
	#[error("Compiler error: {0}")]
	CompilerError(#[from] ExternError),
}
