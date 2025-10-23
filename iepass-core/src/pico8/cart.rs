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

pub fn load_cartridge<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, cartridge: &[u8]) -> Result<(), CartridgeParseError> {
	let section_iter = section_iterator(cartridge)?;
	
	for (name, body) in section_iter {
		let name_str = core::str::from_utf8(name).unwrap_or("<invalid utf8>");
		debug!("load_cartridge: Loading cartridge section {} (len: {})", name_str, body.len());
		match name {
			b"__lua__" => { load_lua_section(vm, body)?; },
			b"__gfx__" => { load_gfx_section(vm, body)?; },
			b"__map__" => { load_map_section(vm, body)?; },
			_ => { info!("load_cartridge: Unknown section name {}", name_str); }
		}
	}
	Ok(())
}

fn load_lua_section<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, data: &[u8]) -> Result<(), CartridgeParseError> {
	let lua_text = core::str::from_utf8(data);
	match lua_text {
		Ok(lua_text) => {
			vm.load(lua_text.as_bytes());
			debug!("load_cartridge: Loaded lua text (len: {})", lua_text.len());
		}
		Err(_) => {
			return Err(CartridgeParseError::InvalidLuaUnicode);
		}
	}
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

fn load_gfx_section<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, data: &[u8]) -> Result<(), CartridgeParseError> {
	let gfx_base_addr = vm.env().memory.base_addr_gfx() as usize;
	let mut max_offset = 0;
	
	for (offset, byte) in split_nonempty_lines(data, 128)
		.flat_map(|(line_idx, line)|
			nibble_chunks(line).take(64).map(|(h, l)| (hex_char_to_nibble(l).unwrap_or(0) << 4) | hex_char_to_nibble(h).unwrap_or(0))
			                   .enumerate().map(move |(col_idx, byte)| (line_idx * 64 + col_idx, byte))
		)
	{
		vm.env().memory[gfx_base_addr + offset] = byte;
		if offset > max_offset { max_offset = offset; }
	}
	
	debug!("load_cartridge: GFX section loaded: {} bytes written to memory starting at 0x{:04x}", max_offset, gfx_base_addr);
	Ok(())
}

fn load_map_section<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, data: &[u8]) -> Result<(), CartridgeParseError> {
	let map_base_addr = vm.env().memory.base_addr_map() as usize;
	let mut max_offset = 0;
	
	for (offset, byte) in split_nonempty_lines(data, 32)
		.flat_map(|(line_idx, line)|
			nibble_chunks(line).take(128).map(|(h, l)| {
				if let (Some(h), Some(l)) = (hex_char_to_nibble(h), hex_char_to_nibble(l)) {
					(h << 4) | l
				}else{
					0
				}
			}).enumerate().map(move |(col_idx, byte)| (line_idx * 128 + col_idx, byte))
		)
	{
		vm.env().memory[map_base_addr + offset] = byte;
		if offset > max_offset { max_offset = offset; }
	}
	
	debug!("load_cartridge: MAP section loaded: {} bytes written starting at 0x{:04x}", max_offset, map_base_addr);
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
		if let Err(err) = load_cartridge(&mut vm, b"Test\nastasdasdasdasdasdsadasdd\n__test__\nasdtest") {
			assert!(false, "Cartridge load error: {}", err);
		}
		// todo: add testing of memory
	}

	#[test]
	fn test_section_iterator_comprehensive() {
		// todo: ai test - remove
		// Test 1: Basic multi-section parsing
		let cartridge = b"pico-8 cartridge // http://www.pico-8.com\nversion 8\n\n__lua__\nprinth(\"test\")\nfor i=1,10 do end\n\n__gfx__\n00112233445566778899aabbccddeeff\n\n__map__\n0102030405060708\n";
		
		let result = section_iterator(cartridge);
		assert!(result.is_ok(), "Should parse valid cartridge");
		
		let sections: Vec<_> = result.unwrap().collect();
		assert_eq!(sections.len(), 3, "Should find 3 sections, got: {}", sections.len());
		
		// Validate lua section
		let (lua_header, lua_body) = &sections[0];
		assert_eq!(core::str::from_utf8(lua_header).unwrap(), "__lua__");
		let lua_content = core::str::from_utf8(lua_body).unwrap();
		assert!(lua_content.contains("printh") && lua_content.contains("for"), 
			"Lua should contain expected code patterns");
		
		// Validate gfx section has hex data
		let (gfx_header, gfx_body) = &sections[1];
		assert_eq!(core::str::from_utf8(gfx_header).unwrap(), "__gfx__");
		let hex_chars = gfx_body.iter().filter(|&&b| b.is_ascii_hexdigit()).count();
		assert!(hex_chars > 20, "Graphics should have substantial hex data");
		
		// Validate map section
		let (map_header, _) = &sections[2];
		assert_eq!(core::str::from_utf8(map_header).unwrap(), "__map__");

	}
}

