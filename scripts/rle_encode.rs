//! ```cargo
//! [dependencies]
//! iepass-core = { path = "../iepass-core", features = ["std"] }
//! ```

use std::fs::File;
use iepass_core::rle;

fn main() {
	let args: Vec<_> = std::env::args().collect();
	
	if let [_, input, output] = args.as_slice() {
		println!("RLE Encoding {input} -> {output}");
		std::io::copy(
			&mut File::open(input).expect("Failed to create output file"),
			&mut rle::Encoder::new_std(&mut File::create(output).expect("Failed to open input file")),
		).unwrap();
	} else {
		eprintln!("Usage: rle_encode <input file> <output file>");
		std::process::exit(1);
	}
}
