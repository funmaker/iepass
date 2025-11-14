use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Once;
use colored::{Color, Colorize};
use colored::control::SHOULD_COLORIZE;
use p8rs::vm::palette;

use crate::{runner, TMP_DIR};
use crate::runner::Log;
use crate::utils::{replace, CollectArray};

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
		
		let test_name = pico8_log.and_then(Log::name)
		                         .or(p8rs_log.and_then(Log::name))
		                         .map(|name| format!(" {}", name))
		                         .unwrap_or("".into());
		
		if let (Some(Log::SCR(pico8_name, _)), Some(Log::SCR(p8rs_name, _))) = (pico8_log, p8rs_log) && pico8_name == p8rs_name {
			try_print_src(&pico8.logs, i, "PICO-8 Screen");
			try_print_src(&p8rs.logs, i, "P8RS Screen");
		}
		
		match (pico8_log, p8rs_log) {
			(Some(pico8_log), Some(p8rs_log)) => panic!("Test{} failed, log mismatch.\n\tpico8 log:\n\t\t{}\n\tp8rs log:\n\t\t{}", test_name, pico8_log, p8rs_log),
			(Some(pico8_log), None)           => panic!("Test{} failed, log mismatch.\n\tpico8 log:\n\t\t{}\n\tp8rs log:\n\t\tNone", test_name, pico8_log),
			(None, Some(p8rs_log))            => panic!("Test{} failed, log mismatch.\n\tpico8 log:\n\t\tNone\n\tp8rs log:\n\t\t{}", test_name, p8rs_log),
			_ => unreachable!()
		}
	}
}

fn try_print_src(logs: &[Log], idx: usize, screen_name: &str) -> Option<Box<[[u8; 128]; 128]>> {
	let test_name = match logs.get(idx) {
		Some(Log::SCR(log_name, _)) => log_name,
		_ => return None,
	};
	
	let start = logs[0..idx].iter()
	                        .rposition(|log| !matches!(log, Log::SCR(n, _) if n == test_name))
	                        .map_or(0, |idx| idx + 1);
	
	let pal = match &logs[start] {
		Log::SCR(n, pal) if n == test_name => pal.strip_prefix("pal | "),
		_ => return None,
	}?;
	
	let pal: [_; 16] =
		pal.as_bytes()
		   .chunks(2)
		   .map(|col| u8::from_ascii_radix(col, 16))
		   .collect_array()
		   .ok()?
		   .try_map(Result::ok)?;
	
	let mut output = vec![];
	for row in 0..128 {
		let line = match &logs[start + 1 + row] {
			Log::SCR(n, pal) if n == test_name => pal.split_once(" | ").map(|(_, data)| data),
			_ => return None,
		}?;
		
		output.push(
			line.as_bytes()
			    .iter()
			    .map(|col| u8::from_ascii_radix(&[*col], 16))
				.array_chunks()
				.flat_map(|[a, b]| [b, a])
			    .collect_array()
			    .ok()?
				.try_map(Result::ok)?
				.map(|col| pal[col as usize])
		);
	}
	
	println!("{:^134}", format!("{screen_name} - {test_name}"));
	print!("   {:<16}", "");
	let digits = "0123456789ABCDEF";
	for digit in digits.chars().take(8).skip(1) {
		print!("{digit:<16}")
	}
	println!();
	print!("   ");
	for _ in 0..8 {
		print!("{digits}");
	}
	println!();
	
	print!("  ╭");
	for _ in 0..128 {
		print!("─")
	}
	println!("╮");
	
	for (row, [upper, lower]) in output.iter().array_chunks().enumerate() {
		let row = row * 2;
		let label = if row % 16 == 0 && row > 0 {
			format!("{:02X}", row)
		} else {
			format!("{:X}", row % 16)
		};
		
		if (idx - start) == row + 1 || (idx - start) == row + 2 {
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
				let uc = palette::color_from_index(upper[x]).rgb();
				let lc = palette::color_from_index(lower[x]).rgb();
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
		println!("│")
	}
	
	print!("  ╰");
	for _ in 0..128 {
		print!("─")
	}
	println!("╯");
	println!();
	
	Some(output.into_boxed_slice().into_array().unwrap())
}

