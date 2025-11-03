use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Once;

use crate::{runner, TMP_DIR};
use crate::runner::Log;
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
	let tmp_cart_path =
		PathBuf::from(TMP_DIR)
			.join(orig_cart_path.as_os_str()
			                    .to_string_lossy()
			                    .trim_suffix(".p8")
			                    .replace(|c: char| !c.is_ascii_alphanumeric(), "_") + ".p8");
	
	fs::write(&tmp_cart_path, &cart).expect("Unable to write tmp cart file");
	
	let pico8 = runner::pico8::run(&tmp_cart_path);
	let p8rs = runner::p8rs::run(&cart);
	
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
		
		match (pico8_log, p8rs_log) {
			(Some(pico8_log), Some(p8rs_log)) => panic!("Test{} failed, log mismatch.\n\tpico8 log:\n\t\t{}\n\tp8rs log:\n\t\t{}", test_name, pico8_log, p8rs_log),
			(Some(pico8_log), None)           => panic!("Test{} failed, log mismatch.\n\tpico8 log:\n\t\t{}\n\tp8rs log:\n\t\tNone", test_name, pico8_log),
			(None, Some(p8rs_log))            => panic!("Test{} failed, log mismatch.\n\tpico8 log:\n\t\tNone\n\tp8rs log:\n\t\t{}", test_name, p8rs_log),
			_ => unreachable!()
		}
	}
}

