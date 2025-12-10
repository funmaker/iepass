use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;
use elf::ElfBytes;
use elf::endian::AnyEndian;
use rustc_demangle::demangle;
use crate::utils::format_bytes;

pub const MAPPINGS: [Mapping; 11] = [
	Mapping::new(0x3C00_0000, 1024 * 1024, MemoryKind::FLASH,   Bus::Data,    0 * 1024), // External Flash
	Mapping::new(0x3FF0_0000,  128 * 1024, MemoryKind::ROM,     Bus::Data,  128 * 1024), // Internal ROM 1
	Mapping::new(0x3FC8_8000,  416 * 1024, MemoryKind::SRAM,    Bus::Data,   32 * 1024), // Internal SRAM 1
	Mapping::new(0x3FCF_0000,   64 * 1024, MemoryKind::SRAM,    Bus::Data,  448 * 1024), // Internal SRAM 2
	
	Mapping::new(0x4000_0000,  256 * 1024, MemoryKind::ROM,     Bus::Inst,    0 * 1024), // Internal ROM 0
	Mapping::new(0x4004_0000,  128 * 1024, MemoryKind::ROM,     Bus::Inst,  128 * 1024), // Internal ROM 1
	Mapping::new(0x4037_0000,   32 * 1024, MemoryKind::SRAM,    Bus::Inst,    0 * 1024), // Internal SRAM 0
	Mapping::new(0x4037_8000,  416 * 1024, MemoryKind::SRAM,    Bus::Inst,   32 * 1024), // Internal SRAM 1
	Mapping::new(0x4200_0000, 1024 * 1024, MemoryKind::FLASH,   Bus::Inst,    0 * 1024), // External Flash
	
	Mapping::new(0x5000_0000,    8 * 1024, MemoryKind::RTCSlow, Bus::Both,    0 * 1024), // RTC SLOW Memory
	Mapping::new(0x600F_E000,    8 * 1024, MemoryKind::RTCFast, Bus::Both,    0 * 1024), // RTC FAST Memory
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MemoryKind {
	ROM,
	SRAM,
	RTCSlow,
	RTCFast,
	FLASH,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Bus {
	Data,
	Inst,
	Both,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MemoryDesc {
	pub kind: MemoryKind,
	pub size: usize,
}

impl MemoryDesc {
	const fn new(kind: MemoryKind, size: usize) -> Self {
		Self { kind, size }
	}
	
	pub const fn get(kind: MemoryKind) -> Self {
		match kind {
			MemoryKind::ROM      => MemoryDesc::new(kind,  384 * 1024),
			MemoryKind::SRAM     => MemoryDesc::new(kind,  512 * 1024),
			MemoryKind::RTCSlow  => MemoryDesc::new(kind,    8 * 1024),
			MemoryKind::RTCFast  => MemoryDesc::new(kind,    8 * 1024),
			MemoryKind::FLASH    => MemoryDesc::new(kind, 1024 * 1024),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Mapping {
	pub address: usize,
	pub size: usize,
	pub kind: MemoryKind,
	pub bus: Bus,
	pub offset: usize,
}

impl Mapping {
	const fn new(address: usize, size: usize, kind: MemoryKind, bus: Bus, offset: usize) -> Self {
		Self { address, size, kind, bus, offset }
	}
}

#[derive(Debug, Clone)]
pub struct Symbol {
	pub name: String,
	pub memory: MemoryKind,
	pub mapping: Mapping,
	pub offset: usize,
	pub address: usize,
	pub size: usize,
}

impl Symbol {
	fn new(name: &str, address: usize, size: usize) -> Option<Self> {
		MAPPINGS.iter()
		        .copied()
		        .find(|mapping| address >= mapping.address && address < mapping.address + mapping.size)
		        .map(|mapping| Symbol {
			        name: format!("{}\n0x{:08x} ({})", demangle(name), address, format_bytes(size as f64)),
			        memory: mapping.kind,
			        mapping,
			        offset: address - mapping.address + mapping.offset,
			        address,
			        size,
		        })
	}
	
	fn from_symbol(name: &str, sym: elf::symbol::Symbol) -> Option<Self> {
		let name = demangle(name).to_string();
		let address = sym.st_value as usize;
		let size = sym.st_size as usize;
		if address == 0 || size == 0 {
			return None;
		}
		
		Self::new(&name, address, size)
	}
}

pub type Symbols = HashMap<MemoryKind, HashMap<Mapping, Vec<Symbol>>>;

pub fn get_symbols(path: impl AsRef<Path>) -> Result<Symbols> {
	let file = fs::read(path)?;
	let elf = ElfBytes::<AnyEndian>::minimal_parse(&file)?;
	
	let mut symbols = HashMap::new();
	let (sections, strtab) = elf.section_headers_with_strtab()?;
	
	let stack = sections.iter()
	                    .flat_map(|sections| sections.iter())
	                    .find(|section| strtab.is_some_and(|strtab| strtab.get(section.sh_name as usize).is_ok_and(|name| name == ".stack")))
	                    .and_then(|section| Symbol::new("Stack", section.sh_addr as usize, section.sh_size as usize));
	
	let symbol_table = elf.symbol_table()?;
	let symbol_iter =
		symbol_table.iter()
		            .flat_map(|(symtab, strtab)|
			            symtab.iter()
			                  .flat_map(|sym| Symbol::from_symbol(strtab.get(sym.st_name as usize).unwrap(), sym))
		            )
		            .chain(fixed_symbols())
		            .chain(stack);
	
	for symbol in symbol_iter {
		symbols.entry(symbol.memory)
		       .or_insert(HashMap::new())
		       .entry(symbol.mapping)
		       .or_insert(Vec::new())
		       .push(symbol);
	}
	
	Ok(symbols)
}

fn fixed_symbols() -> impl IntoIterator<Item=Symbol> {
	[
		Symbol::new("IRAM Cache", 0x40370000, 32 * 1024).unwrap(),
		Symbol::new("DRAM Cache", 0x3FCF8000, 32 * 1024).unwrap(),
	]
}
