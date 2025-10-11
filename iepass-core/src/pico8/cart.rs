/*!
 * Pico-8 Cartridge Loading
 * 
 * This module implements parsing and loading of Pico-8 cartridge (.p8) files.
 * 
 * ## Cartridge Format
 * 
 * A Pico-8 cartridge file starts with a header and contains multiple sections:
 * 
 * ```text
 * pico-8 cartridge // http://www.pico-8.com
 * version 8
 * 
 * __lua__
 * [Lua source code]
 * 
 * __gfx__
 * [Graphics data as hexadecimal]
 * 
 * __map__
 * [Map/tilemap data as hexadecimal]
 * 
 * __sfx__
 * [Sound effects data]
 * 
 * __music__
 * [Music pattern data]
 * ```
 * 
 * ## Usage
 * 
 * ```rust
 *  // todo
 * ```
 */

use crate::pico8::Pico8VM;
use core::alloc::Allocator;
use thiserror::Error;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct SectionIterator<'a, Lines> {
	pub cart: &'a [u8],
	lines: Lines,
	last_header: Option<&'a [u8]>,
}

pub fn section_iterator<'a>(cart: &'a [u8]) -> Result<SectionIterator<'a, impl Iterator<Item=&'a [u8]> + 'a>, CartridgeParseError> {
	let mut lines = cart.split(|x: &u8| *x == b'\n' || *x == b'\r')
	                    .filter(|line| line.starts_with(b"__") && line.ends_with(b"__"));
	
	let last_header = lines.next();
	
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
		lines,
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
		
		let next_header = self.lines.next();
		
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

pub fn load_cartridge<A: Allocator + Clone + 'static>(_vm: &mut Pico8VM<A>, cartridge: &[u8]) -> Result<(), CartridgeParseError> {
	let section_iter = section_iterator(cartridge);
	
	match section_iter {
		Ok(section_iter) => {
			for (name, body) in section_iter {
				let name_str = core::str::from_utf8(name).unwrap_or("<invalid utf8>"); 
				debug!("load_cartridge: Loading cartridge section {} (len: {})", name_str, body.len());
				match name {
					b"__gfx__" => {
						load_gfx_section(_vm, body)?;
					},
					b"__lua__" => {
						let lua_text = core::str::from_utf8(body);
						match lua_text {
							Ok(lua_text) => {
								_vm.load(lua_text.as_bytes());
								debug!("load_cartridge: Loaded lua text (len: {})", lua_text.len());
							}
							Err(_) => {
								return Err(CartridgeParseError::InvalidLuaUnicode);
							}
						}
					},
					_ => {
						info!("load_cartridge: Unknown section name {}", name_str);
					}
				}
			}
			Ok(())
		},
		Err(e) => {
			Err(e)
		}
	}
}

fn load_gfx_section<A: Allocator + Clone + 'static>(vm: &mut Pico8VM<A>, data: &[u8]) -> Result<(), CartridgeParseError> {
	let gfx_base_addr = vm.env().memory.base_addr_gfx() as usize;
	let memory_offset = 0; // sprite sheet
	
	let lines = data.split(|&b| b == b'\n' || b == b'\r');
	
	for (line_idx, line) in lines.enumerate() {
		if line.is_empty() { continue; }
		
		// todo: implement
		
		if line_idx < 5 || line_idx % 32 == 0 {
			debug!("GFX line {}", line_idx);
		}
	}
	
	debug!("GFX section loaded: {} bytes written to memory starting at 0x{:04x}", memory_offset, gfx_base_addr);
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
	fn test_gfx_section_loading() {
		// todo: ai test - remove
		// Test GFX section with known data pattern
		let cartridge = b"pico-8 cartridge // http://www.pico-8.com\nversion 8\n\n__gfx__\n0123456789abcdef0f0e0d0c0b0a09080706050403020100fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeede\n1010101010101010202020202020202030303030303030304040404040404040505050505050505060606060\n";
		
		let mut vm = Pico8VM::new().unwrap();
		let result = load_cartridge(&mut vm, cartridge);
		assert!(result.is_ok(), "Should successfully load cartridge with GFX data");
		
		// Verify that GFX data was written to memory
		let gfx_base = vm.env().memory.base_addr_gfx() as usize;
		
		// Check first few bytes to verify nibble swapping is correct
		// First hex pair "01" should be stored as 0x10 (nibbles swapped)
		assert_eq!(vm.env().memory[gfx_base], 0x10, "First byte should have swapped nibbles");
		
		// Second hex pair "23" should be stored as 0x32
		assert_eq!(vm.env().memory[gfx_base + 1], 0x32, "Second byte should have swapped nibbles");
		
		// Check that some data was actually written
		let mut non_zero_count = 0;
		for i in 0..64 {
			if vm.env().memory[gfx_base + i] != 0 {
				non_zero_count += 1;
			}
		}
		assert!(non_zero_count > 10, "Should have loaded substantial GFX data, found {} non-zero bytes", non_zero_count);
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

