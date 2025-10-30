use core::alloc::Allocator;
use thiserror::Error;

use crate::pico8::Pico8VM;

#[derive(Debug)]
pub struct SectionIterator<'a, Lines> {
	pub cart: &'a [u8],
	header_lines: Lines,
	last_header: Option<&'a [u8]>,
}

pub fn section_iterator<'a>(cart: &'a [u8]) -> Result<SectionIterator<'a, impl Iterator<Item=&'a [u8]> + 'a>, CartridgeParseError> {
	let mut header_lines = cart.split(|x: &u8| *x == b'\n' || *x == b'\r')
	                           .filter(|line| line.starts_with(b"__") && line.ends_with(b"__"));
	
	let last_header = header_lines.next();
	
	if last_header.is_none() {
		return Err(CartridgeParseError::NoDataSection);
	}
	
	let file_header = &cart[0..cart.subslice_range(last_header.unwrap()).unwrap().start];
	
	if file_header.len() < 25 {
		info!("SectionIterator: File header too short");
		return Err(CartridgeParseError::InvalidHeader);
	}
	
	// todo: check header better
	
	Ok(SectionIterator {
		cart,
		header_lines,
		last_header,
	})
}

impl<'a, Lines> Iterator for SectionIterator<'a, Lines>
where Lines: Iterator<Item=&'a [u8]> + 'a {
	type Item = (&'a [u8], &'a [u8]);
	
	fn next(&mut self) -> Option<Self::Item> {
		let header = self.last_header;
		
		if header.is_none() {
			return None;
		}
		
		let header = header.unwrap();
		
		let next_header = self.header_lines.next();
		
		let body_start = self.cart.subslice_range(header).unwrap().end;
		let body_end = next_header.and_then(|header| self.cart.subslice_range(header))
		                               .map(|range| range.start)
		                               .unwrap_or(self.cart.len());
		
		self.last_header = next_header;
		
		Some((
			header,
			&self.cart[body_start..body_end],
		))
	}
}

struct CartLoadContext {
	lua_code: Vec<u8>,
	gfx_loaded: bool,
	map_loaded: bool,
	long_map_loaded: bool,
}

pub fn load_cartridge<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, cartridge: &[u8]) -> Result<(), CartridgeParseError> {
	let section_iter = section_iterator(cartridge)?;
	let mut load_ctx = CartLoadContext {
		lua_code: Vec::new(),
		gfx_loaded: false,
		map_loaded: false,
		long_map_loaded: false,
	};
	
	for (name, body) in section_iter {
		let name_str = core::str::from_utf8(name).unwrap_or("<invalid utf8>");
		debug!("load_cartridge: Loading cartridge section {} (len: {})", name_str, body.len());
		match name {
			b"__lua__" => { load_lua_section(vm, body, &mut load_ctx)?; },
			b"__gfx__" => { load_gfx_section(vm, body, &mut load_ctx)?; },
			b"__map__" => { load_map_section(vm, body, &mut load_ctx)?; },
			_ => { info!("load_cartridge: Unknown section name {}", name_str); }
		}
	}

	if !load_ctx.lua_code.is_empty() {
		let lua_text = core::str::from_utf8(&load_ctx.lua_code);
		match lua_text {
			Ok(lua_text) => {
				vm.load(lua_text.as_bytes());
				debug!("load_cartridge: Loaded lua text (len: {})", lua_text.len());
			}
			Err(_) => {
				return Err(CartridgeParseError::InvalidLuaUnicode);
			}
		}
	}

	Ok(())
}

fn load_lua_section<A: Allocator + Clone + 'static>(_vm: &mut Pico8VM<A>, data: &[u8], load_ctx: &mut CartLoadContext) -> Result<(), CartridgeParseError> {
	load_ctx.lua_code.extend_from_slice(data);
	load_ctx.lua_code.push(b'\n');
	Ok(())
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

fn load_gfx_section<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, data: &[u8], load_ctx: &mut CartLoadContext) -> Result<(), CartridgeParseError> {
	let gfx_base_addr = vm.env().memory.base_addr_gfx() as usize;
	let mut max_offset = -1;
	
	if load_ctx.gfx_loaded {
		debug!("load_cartridge: GFX section was already loaded, overwriting data.");
	}
	
	for (offset, byte) in split_nonempty_lines(data, 128) // todo: figure out pico8 behaviour, if load_ctx.long_map_loaded { 64 } else { 128 }
		.flat_map(|(line_idx, line)|
			nibble_chunks(line).take(64).map(|(h, l)| (hex_char_to_nibble(l).unwrap_or(0) << 4) | hex_char_to_nibble(h).unwrap_or(0))
			                   .enumerate().map(move |(col_idx, byte)| ((line_idx * 64 + col_idx) as i16, byte))
		)
	{
		vm.env().memory[gfx_base_addr + offset as usize] = byte;
		if offset > max_offset { max_offset = offset; }
	}
	
	load_ctx.gfx_loaded = true;
	
	if max_offset == -1 {
		debug!("load_cartridge: GFX section loaded: empty section!");
	} else {
		debug!("load_cartridge: GFX section loaded: 0x{:X} bytes written to memory starting at 0x{:04x}", max_offset + 1, gfx_base_addr);
	}
	
	Ok(())
}

fn load_map_section<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, data: &[u8], load_ctx: &mut CartLoadContext) -> Result<(), CartridgeParseError> {
	let map_base_addr = vm.env().memory.base_addr_map() as usize;
	let gfx_base_addr = vm.env().memory.base_addr_gfx() as usize;
	let mut max_offset = -1;
	
	if load_ctx.map_loaded {
		debug!("load_cartridge: MAP section was already loaded, overwriting data.")
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
		vm.env().memory[addr] = byte;
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
	// 			vm.env().memory[addr] = 0;
	// 		}
	// 	}
	// }
	
	if max_offset == -1 {
		debug!("load_cartridge: MAP section loaded: empty section!");
	}else{
		debug!("load_cartridge: MAP section loaded: 0x{:X} bytes written starting at 0x{:04x}", max_offset, map_base_addr);
	}
	
	Ok(())
}



#[derive(Debug, Clone)]
pub struct CartridgeSection<'a> {
	pub name: &'a [u8],
	pub content: &'a [u8],
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
pub enum CartridgeParseError {
	#[error("Could not load cartridge, invalid input data.")] 
	InvalidInputData,
	#[error("Invalid cartridge header, expected 'pico-8 cartridge'")]
	InvalidHeader,
	#[error("Could not load cartridge, no data sections detected.")]
	NoDataSection,
	#[error("Could not load cartridge, lua code contains invalid UTF-8 characters.")]
	InvalidLuaUnicode,
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec::Vec;
	
	#[test]
	fn test_load_cartridge() {
		let mut vm = Pico8VM::new().unwrap();
		if let Err(err) = load_cartridge(&mut vm, b"pico-8 cartridge // http://www.pico-8.com
version 43
__lua__
__gfx__
a0aa00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006543217
b0bb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006543
c0cc0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000654321

__map__
a0aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006543217
b0bb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006543
c0cc000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000890000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000654321
") {
			assert!(false, "Cartridge load error: {}", err);
		}
		
		let result = vm.run();
		
		
		let env = vm.env.clone();
		let mem = &env.borrow().memory;
		
		// test GFX loading
		assert_eq!(mem[0x3e], 0x34);
		assert_eq!(mem[0x3f], 0x12);
		assert_eq!(mem[0x40], 0x0b);
		assert_eq!(mem[0x41], 0xbb);
		assert_eq!(mem[0x7e], 0x34);
		assert_eq!(mem[0x7f], 0x00);
		assert_eq!(mem[0x80], 0x0c);
		assert_eq!(mem[0x81], 0xcc);
		
		// test MAP loading
		assert_eq!(mem[0x207f], 0x21);
		assert_eq!(mem[0x2080], 0xb0);
		assert_eq!(mem[0x2081], 0xbb);
		assert_eq!(mem[0x20fe], 0x43);
		assert_eq!(mem[0x20ff], 0x00);
		assert_eq!(mem[0x2100], 0xc0);
		assert_eq!(mem[0x2101], 0xcc);
	}
}

