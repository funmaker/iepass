use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Once;
use colored::{Color, Colorize};
use colored::control::SHOULD_COLORIZE;
use p8rs::vm::palette;

use crate::{runner, TMP_DIR};
use crate::log::Log;
use crate::utils::replace;

static SETUP: Once = Once::new();
fn setup() {
	SETUP.call_once(|| {
		match fs::remove_dir_all(TMP_DIR) {
			Ok(_) => {},
			Err(err) if err.kind() == ErrorKind::NotFound => {},
			err => err.expect("Unable to remove tmp dir"),
		}
		fs::create_dir_all(TMP_DIR).expect("Unable to create tmp dir");
	});
}

pub fn test_cartridge(path: impl AsRef<Path>) {
	setup();
	
	println!();
	println!("=== {} ===", path.as_ref().display());
	
	let orig_cart_path = path.as_ref();
	let cart = fs::read(orig_cart_path).expect("Unable to read cart file");
	let cart = replace(
		&cart,
		b"__lua__",
		concat!(
			"__lua__\n",
			include_str!("polyfill.lua"),
			"\n"
		).as_bytes(),
	);
	let sanitized_name = orig_cart_path.as_os_str()
	                                   .to_string_lossy()
	                                   .trim_suffix(".p8")
	                                   .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
	let tmp_cart_path = PathBuf::from(TMP_DIR).join(format!("{sanitized_name}.p8"));
	let pico8_log_path = PathBuf::from(TMP_DIR).join(format!("{sanitized_name}.pico8.log"));
	let p8rs_log_path = PathBuf::from(TMP_DIR).join(format!("{sanitized_name}.p8rs.log"));
	
	println!("pico8 log: {}", pico8_log_path.display());
	println!("p8rs  log: {}", p8rs_log_path.display());
	println!();
	
	fs::write(&tmp_cart_path, &cart).expect("Unable to write tmp cart file");
	
	let pico8 = runner::pico8::run(&tmp_cart_path, &pico8_log_path);
	let p8rs = runner::p8rs::run(&cart, &p8rs_log_path);
	
	match (pico8.timeout, p8rs.timeout) {
		(true, true) => panic!("pico8 and p8rs timeout"),
		(true, false) => panic!("pico8 timeout"),
		(false, true) => panic!("p8rs timeout"),
		_ => {}
	}
	
	match (pico8.runtime_error, p8rs.runtime_error) {
		(Some(pico8_err), None) => panic!("pico8 raised a runtime error, but p8rs did not.\n\tpico8 runtime error:\n\t\t{}\n\tp8rs runtime error:\n\t\tNone", pico8_err),
		(None, Some(p8rs_err)) => panic!("p8rs raised a runtime error, but pico8 did not.\n\tpico8 runtime error:\n\t\tNone\n\tp8rs runtime error:\n\t\t{}", p8rs_err),
		_ => {}
	}
	
	for i in 0..usize::max(pico8.logs.len(), p8rs.logs.len()) {
		let pico8_log = pico8.logs.get(i);
		let p8rs_log = p8rs.logs.get(i);
		if pico8_log == p8rs_log {
			continue
		}
		
		let test_name = pico8_log.and_then(|log| log.name()).unwrap_or("<unknown>");
		let p8rs_name = p8rs_log.and_then(|log| log.name()).unwrap_or("<unknown>");
		
		if test_name != p8rs_name {
			panic!("Test failed, log name mismatch. (pico8: {test_name}, p8rs: {p8rs_name})");
		}
		
		match (pico8_log, p8rs_log) {
			(
				Some(Log::TEST(_, pico8_val)),
				Some(Log::TEST(_, p8rs_val))
			) => {
				panic!("Test {test_name} failed, value mismatch. (pico8: {pico8_val}, p8rs: {p8rs_val})");
			},
			(
				Some(Log::MEM(pico8_name, pico8_offset, pico8_data)),
				Some(Log::MEM(p8rs_name, p8rs_offset, p8rs_data))
			) => {
				println!("pico8 {pico8_name} memory at 0x{pico8_offset:04x}: {}", to_hexstring(pico8_data));
				println!("p8rs  {p8rs_name} memory at 0x{p8rs_offset:04x}: {}", to_hexstring(p8rs_data));
				
				if pico8_offset != p8rs_offset {
					panic!("Test {test_name} failed, memory offset mismatch. (pico8: {pico8_offset}, p8rs: {p8rs_offset})");
				} else if pico8_data.len() != p8rs_data.len() {
					panic!("Test {test_name} failed, memory size mismatch. (pico8: {}, p8rs: {})", pico8_data.len(), p8rs_data.len());
				} else {
					let offset = pico8_data.iter().zip(p8rs_data).position(|(a, b)| a != b).unwrap();
					let position = pico8_offset.wrapping_add(offset as u16);
					panic!("Test {test_name} failed, memory mismatch at 0x{position:04x}. (pico8: 0x{:02x}, p8rs: 0x{:02x})", pico8_data[offset], p8rs_data[offset]);
				}
			},
			(
				Some(Log::SCR(pico8_name, pico8_pal, pico8_pixels)),
				Some(Log::SCR(p8rs_name, p8rs_pal, p8rs_pixels))
			) => {
				let pixel_pos =
					pico8_pixels.iter()
					            .zip(p8rs_pixels.iter())
					            .enumerate()
					            .flat_map(|(y, (a, b))|
						            a.iter()
						             .zip(b.iter())
						             .position(|(a, b)| a != b)
						             .map(|x| (x, y)))
					            .next();
				
				print_scr("pico8 Screen", pico8_name, pico8_pal, pico8_pixels, pixel_pos);
				print_scr("p8rs  Screen", p8rs_name, p8rs_pal, p8rs_pixels, pixel_pos);
				
				if pico8_pal != p8rs_pal {
					panic!("Test {test_name} failed, palette mismatch. (pico8: {pico8_pal:?}, p8rs: {p8rs_pal:?})");
				} else {
					let (col, row) = pixel_pos.unwrap();
					panic!("Test {test_name} failed, pixel mismatch at {col}x{row}. (pico8: {:x}, p8rs: {:x})", pico8_pixels[row][col], p8rs_pixels[row][col]);
				}
			},
			(
				Some(Log::OTHER(pico8_text)),
				Some(Log::OTHER(p8rs_text))
			) => {
				panic!("Test {test_name} failed, log mismatch. (pico8: {pico8_text}, p8rs: {p8rs_text})")
			},
			(pico8_log, p8rs_log) => panic!("Test {test_name} failed, log mismatch. (pico8: {}, p8rs: {})", pico8_log.map_or("None" ,Log::kind), p8rs_log.map_or("None" ,Log::kind)),
		}
	}
}

fn to_hexstring(data: &[u8]) -> String {
	use std::fmt::Write;
	
	let mut ret = String::with_capacity(data.len() * 2 + data.len() / 4);
	
	for chunk in data.chunks(4) {
		for byte in chunk {
			write!(ret, "{:02x}", byte).unwrap();
		}
		write!(ret, " ").unwrap();
	}
	
	ret.truncate(ret.len().saturating_sub(1));
	
	ret
}

fn print_scr(screen_name: &str, test_name: &str, pal: &[u8; 16], pixels: &[[u8; 128]; 128], error_pos: Option<(usize, usize)>) {
	let (error_col, error_row) = error_pos.unzip();
	
	println!("{:^134}", format!("{screen_name} - {test_name}"));
	print!("   {:<16}", "");
	let digits = "0123456789ABCDEF";
	for digit in digits.chars().take(8).skip(1) {
		print!("{digit:<16}")
	}
	println!();
	print!("   ");
	for (col, char) in (0..128).zip(digits.chars().cycle()) {
		if Some(col) == error_col {
			print!("{}", char.to_string().color(Color::Black).on_color(Color::Red));
		} else {
			print!("{char}")
		}
	}
	println!();
	
	print!("  ╭");
	for col in 0..128 {
		if Some(col) == error_col {
			print!("╳");
		} else {
			print!("─")
		}
	}
	println!("╮");
	
	for (row, [upper, lower]) in pixels.iter().array_chunks().enumerate() {
		let row = row * 2;
		let is_row_error = error_row == Some(row) || error_row == Some(row + 1);
		let label = if row % 16 == 0 && row > 0 {
			format!("{:02X}", row)
		} else {
			format!("{:X}", row % 16)
		};
		
		if is_row_error {
			print!("{:>2}╳", label.color(Color::Black).on_color(Color::Red));
		} else {
			print!("{:>2}│", label);
		}
		
		for x in 0..128 {
			let grid = match (row % 16 == 0 && row > 0, x % 16 == 0 && x > 0) {
				(true, true) => Some("┼"),
				(true, false) => Some("┄"),
				(false, true) => Some("┊"),
				_ => None
			};
			
			if SHOULD_COLORIZE.should_colorize() {
				let uc = palette::color_from_index(pal[upper[x] as usize & 0x0F]).rgb();
				let lc = palette::color_from_index(pal[lower[x] as usize & 0x0F]).rgb();
				if let Some(grid) = grid && uc == lc {
					let gc = if uc.0 / 3 + uc.1 / 3 + uc.2 / 3 > 128 {
						(uc.0.saturating_sub(64), uc.1.saturating_sub(64), uc.2.saturating_sub(64))
					} else {
						(uc.0.saturating_add(64), uc.1.saturating_add(64), uc.2.saturating_add(64))
					};
					
					print!("{}", grid.truecolor(gc.0, gc.1, gc.2).on_truecolor(lc.0, lc.1, lc.2));
				} else {
					print!("{}", "▀".truecolor(uc.0, uc.1, uc.2).on_truecolor(lc.0, lc.1, lc.2))
				}
			} else {
				match (upper[x] & 0x8F, lower[x] & 0x8F, grid) {
					(0, 0, Some(grid)) => print!("{}", grid),
					(0, 0, _) => print!(" "),
					(_, 0, _) => print!("▀"),
					(0, _, _) => print!("▄"),
					(_, _, _) => print!("█"),
				}
			}
		}
		
		if is_row_error {
			println!("╳");
		} else {
			println!("│");
		}
	}
	
	print!("  ╰");
	for col in 0..128 {
		if Some(col) == error_col {
			print!("╳");
		} else {
			print!("─")
		}
	}
	println!("╯");
	println!();
}

