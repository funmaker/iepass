use std::{env, fs};
use std::fs::ReadDir;
use std::io::Write;
use std::path::PathBuf;

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
	let carts_path = option_env!("CARTS_DIR").unwrap_or("carts");
	let output_path = PathBuf::from(env::var("OUT_DIR").expect("Can't read OUT_DIR env var")).join("generated_tests.rs");
	let carts_dir = fs::read_dir(carts_path).expect("Can't open carts dir");
	let mut output = fs::File::create(&output_path).expect("failed to create generated_tests.rs");
	
	println!("cargo:rerun-if-changed={}", output_path.display());
	traverse(carts_dir, &mut output, 0);
}

fn traverse(dir: ReadDir, output: &mut impl Write, depth: usize) {
	let mut dir: Vec<_> =
		dir.flat_map(|entry| {
			let entry = match entry {
				Ok(entry) => entry,
				Err(err) => {
					warn!("Cannot access dir content: {err}");
					return None;
				}
			};
			
			let filetype = match entry.file_type() {
				Ok(filetype) => filetype,
				Err(err) => {
					warn!("Cannot retrieve filetype of {}: {err}", entry.path().display());
					return None;
				},
			};
			
			Some((filetype, entry.path(), entry.file_name()))
		}).collect();
	
	dir.sort_by(|(t1, p1, _), (t2, p2, _)|
		t1.is_file()
		  .cmp(&t2.is_file())
		  .then_with(|| p1.cmp(p2))
	);
	
	let max_width = dir.iter()
	                   .filter(|(t, _, _)| t.is_file())
	                   .map(|(_, _, name)| name.len())
	                   .max()
	                   .unwrap_or(0);
	
	for (file_type, path, file_name) in dir {
		if file_type.is_dir() {
			println!("cargo:rerun-if-changed={}", path.display());
			
			let name = clean_file_name(&file_name.to_string_lossy());
			let carts_dir = match fs::read_dir(&path) {
				Ok(dir) => dir,
				Err(err) => {
					warn!("Cannot access {}: {err}", path.display());
					continue;
				}
			};
			
			writeln_ident!(output, depth, "mod {name} {{");
			traverse(carts_dir, output, depth + 1);
			writeln_ident!(output, depth, "}}");
			writeln_ident!(output, depth, "");
		} else if file_type.is_file() {
			let name = match file_name.to_string_lossy().strip_suffix(".p8") {
				Some(name) => clean_file_name(name),
				None => {
					warn!("Unexpected file {}, expected .p8 cartridge", path.display());
					continue;
				}
			};
			
			let blank = "";
			let padding = max_width.saturating_sub(name.len() + 3);
			writeln_ident!(output, depth, "#[test] fn r#{name}(){blank:padding$} {{ crate::tester::test_cartridge({path:?}) }}");
		} else {
			warn!("Unknown filetype {:?} of {}", file_type, path.display());
			continue;
		}
	}
}

fn clean_file_name(filename: &str) -> String {
	filename.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}
