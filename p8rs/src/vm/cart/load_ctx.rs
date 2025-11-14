use alloc::borrow::Cow;
use core::alloc::Allocator;
use thiserror::Error;
use p8rs_piccolo::ExternError;
use p8rs_types::p8scii;

use crate::vm::P8rs;

pub struct CartLoadContext<'vm, 'c, A: Allocator + 'static> {
	pub vm: &'vm mut P8rs<A>,
	pub lua_code: Cow<'c, [u8]>,
	pub gfx_loaded: bool,
	pub map_loaded: bool,
}

impl<'vm, 'c, A: Allocator + 'static> CartLoadContext<'vm, 'c, A> {
	pub fn new(vm: &'vm mut P8rs<A>) -> Self {
		CartLoadContext {
			vm,
			lua_code: Cow::Borrowed(b""),
			gfx_loaded: false,
			map_loaded: false,
		}
	}
	
	pub fn load_section(&mut self, name: &[u8], body: &'c [u8]) -> Result<(), CartLoadError> {
		debug!("load_section: Loading cartridge section {} (len: {})", p8scii::Display(name), body.len());
		
		match name {
			b"__lua__" => self.load_lua_section(body)?,
			b"__gfx__" => self.load_gfx_section(body)?,
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
		let gfx_base_addr = self.vm.runtime.memory.base_addr_gfx() as usize;
		let mut max_offset = -1;
		
		if self.gfx_loaded {
			debug!("load_gfx_section: GFX section was already loaded, overwriting data.");
		}
		
		for (offset, byte) in split_nonempty_lines(data, 128) // todo: figure out pico8 behaviour, if load_ctx.long_map_loaded { 64 } else { 128 }
			.flat_map(|(line_idx, line)|
				nibble_chunks(line).take(64).map(|(h, l)| (hex_char_to_nibble(l).unwrap_or(0) << 4) | hex_char_to_nibble(h).unwrap_or(0))
				                   .enumerate().map(move |(col_idx, byte)| ((line_idx * 64 + col_idx) as i16, byte))
			)
		{
			self.vm.runtime.memory[gfx_base_addr + offset as usize] = byte;
			if offset > max_offset { max_offset = offset; }
		}
		
		self.gfx_loaded = true;
		
		if max_offset == -1 {
			debug!("load_gfx_section: GFX section loaded: empty section!");
		} else {
			debug!("load_gfx_section: GFX section loaded: 0x{:X} bytes written to memory starting at 0x{:04x}", max_offset + 1, gfx_base_addr);
		}
		
		Ok(())
	}
	
	fn load_map_section(&mut self, data: &[u8]) -> Result<(), CartLoadError> {
		let map_base_addr = self.vm.runtime.memory.base_addr_map() as usize;
		let mut max_offset = -1;
		
		if self.map_loaded {
			debug!("load_map_section: MAP section was already loaded, overwriting data.")
		}
		
		for (offset, byte) in split_nonempty_lines(data, 33)
			.flat_map(|(line_idx, line)|
				nibble_chunks(line).take(128).map(|(h, l)| {
					if let (Some(h), Some(l)) = (hex_char_to_nibble(h), hex_char_to_nibble(l)) {
						(h << 4) | l
					}else{
						0
					}
				}).enumerate().map(move |(col_idx, byte)| ((line_idx * 128 + col_idx) as i16, byte))
			)
		{
			if offset >= 0x1000 { max_offset = 0x1000; break; }
			let addr = map_base_addr + offset as usize;
			self.vm.runtime.memory[addr] = byte;
			if offset > max_offset { max_offset = offset; }
		}
		
		// if gfx is loaded and the map section is longer than 0x1000, zero out the remaining shared memory area written by gfx (0x1000..0x2000, shared between MAP and GFX)
		// (Pico8 behavior)
		
		// todo: figure out the behaviour
		// load_ctx.map_loaded = true;
		// if max_offset >= 0x1000 {
		// 	load_ctx.long_map_loaded = true;
		// 
		// 	if load_ctx.gfx_loaded {
		// 		debug!("Clearing extended GFX from 0x{:X}", max_offset);
		// 		for offset in 0x1000..0x2000 {
		// 			let addr = gfx_base_addr + offset;
		// 			vm.runtime.memory[addr] = 0;
		// 		}
		// 	}
		// }
		
		if max_offset == -1 {
			debug!("load_map_section: MAP section loaded: empty section!");
		}else{
			debug!("load_map_section: MAP section loaded: 0x{:X} bytes written starting at 0x{:04x}", max_offset, map_base_addr);
		}
		
		Ok(())
	}
}

fn hex_char_to_nibble(hex_char: u8) -> Option<u8> {
	match hex_char {
		b'0'..=b'9' => Some(hex_char - b'0'),
		b'a'..=b'f' => Some(hex_char - b'a' + 10),
		b'A'..=b'F' => Some(hex_char - b'A' + 10),
		_ => None,
	}
}

fn split_nonempty_lines(data: &[u8], max_lines: usize) -> impl Iterator<Item=(usize, &[u8])> {
	data.split(|&b| b == b'\n' || b == b'\r')
	    .filter(|line| !line.is_empty()).take(max_lines).enumerate()
}

fn nibble_chunks(text: &[u8]) -> impl Iterator<Item=(u8, u8)> + '_ {
	text.chunks(2).map(|chunk| (chunk[0], chunk.get(1).copied().unwrap_or(b'0')))
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
