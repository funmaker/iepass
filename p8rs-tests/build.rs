use std::{env, fs};
use std::fs::ReadDir;
use std::io::Write;
use std::path::PathBuf;

const CARTS_DIR: &str = "carts";

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

macro_rules! writeln_ident {
    ($out:expr, $ident:expr, $($content: tt)*) => {
	    for _ in 0..$ident { write!($out, "\t").expect("Can't write to generated_tests.rs"); }
	    writeln!($out, $($content)*).expect("Can't write to generated_tests.rs");
    };
}

fn main() {
	let output_path = PathBuf::from(env::var("OUT_DIR").expect("Can't read OUT_DIR env var")).join("generated_tests.rs");
	let carts_dir = fs::read_dir(CARTS_DIR).expect("Can't open carts dir");
	let mut output = fs::File::create(&output_path).expect("failed to create generated_tests.rs");
	
	println!("cargo:rerun-if-changed={}", output_path.display());
	traverse(carts_dir, &mut output, 0);
}

fn traverse(dir: ReadDir, output: &mut impl Write, depth: usize) {
	let mut first = true;
	for entry in dir {
		let entry = match entry {
			Ok(entry) => entry,
			Err(err) => {
				warn!("Cannot access dir content: {err}");
				continue;
			}
		};
		
		let filetype = match entry.file_type() {
			Ok(filetype) => filetype,
			Err(err) => {
				warn!("Cannot retrieve filetype of {}: {err}", entry.path().display());
				continue;
			},
		};
		
		if filetype.is_dir() {
			println!("cargo:rerun-if-changed={}", entry.path().display());
			
			let name = clean_file_name(&entry.file_name().to_string_lossy());
			let carts_dir = match fs::read_dir(entry.path()) {
				Ok(dir) => dir,
				Err(err) => {
					warn!("Cannot access {}: {err}", entry.path().display());
					continue;
				}
			};
			
			if !first { writeln_ident!(output, depth, ""); }
			writeln_ident!(output, depth, "mod {name} {{");
			traverse(carts_dir, output, depth + 1);
			writeln_ident!(output, depth, "}}");
			first = false;
		} else if filetype.is_file() {
			println!("cargo:rerun-if-changed={}", entry.path().display());
			
			let name = entry.file_name();
			let name = match name.to_string_lossy().strip_suffix(".p8") {
				Some(name) => clean_file_name(name),
				None => {
					warn!("Unexpected file {}, expected .p8 cartridge", entry.path().display());
					continue;
				}
			};
			let cart_path = entry.path();
			
			if !first { writeln_ident!(output, depth, ""); }
			writeln_ident!(output, depth, "#[test]");
			writeln_ident!(output, depth, "fn {name}() {{");
			writeln_ident!(output, depth + 1, "crate::tester::test_cartridge({cart_path:?})");
			writeln_ident!(output, depth, "}}");
			first = false;
		} else {
			warn!("Unknown filetype {:?} of {}", filetype, entry.path().display());
			continue;
		}
	}
}

fn clean_file_name(filename: &str) -> String {
	filename.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}
