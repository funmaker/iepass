//! ```cargo
//! [dependencies]
//! image = "0.25.8"
//! ```

use std::fs::File;
use std::io::Write;
use image::ImageReader;
use image::Pixel;

const HEADER: [u8; 8] = [
	4, // 16..128 characters widths
	8, // 128..256 characters width
	6, // character height
	0, 0, // draw offset
	0, // font flags (0x1 apply_size_adjustments  0x2: apply tabs relative to cursor home)
	4, // tab width
	0, // unused
];

fn main() {
	let args: Vec<_> = std::env::args().collect();
	
	if let [_, input, output] = args.as_slice() {
		println!("Generating Font {input} -> {output}");
		let image = ImageReader::open(input)
			.expect("Failed to open image")
			.decode()
			.expect("Failed to decode image");
		
		let mut output = File::create(output).expect("Failed to create output file");
		
		if image.width() != 128 || image.height() != 128 {
			eprintln!("Invalid image dimensions. Expected 128x128, got {}x{}", image.width(), image.height());
			return;
		}
		
		let mut data = [0_u8; 2048];
		let chars = data.as_chunks_mut::<8>().0;
		
		chars[0] = HEADER;
		
		let image = image.into_luma8();
		for y in 1..16 {
			for x in 0..16 {
				let char = x + y * 16;
				for py in 0..8 {
					for px in 0..8 {
						let pixel = image.get_pixel(x * 8 + px, y * 8 + py).channels()[0];
						if pixel > 127 {
							chars[char as usize][py as usize] |= 1 << px;
						}
					}
				}
			}
		}
		
		
		write!(&mut output, "\
			// ! GENERATED FILE !\n\
			// !  DO NOT EDIT   !\n\
			//\n\
			// use `cargo make build-font` instead\
			\n\
			\n\
			pub const SYSTEM_FONT: [u8; 2048] = [\n\
		").expect("Failed to write to output file");
		
		for (n, byte) in data.iter().enumerate() {
			if n % 8 == 0 {
				write!(&mut output, "\t").expect("Failed to write to output file");
			}
			write!(&mut output, "{byte},").expect("Failed to write to output file");
			if n % 8 == 7 {
				write!(&mut output, "\n").expect("Failed to write to output file");
			} else {
				write!(&mut output, " ").expect("Failed to write to output file");
			}
		}
		
		write!(&mut output, "];").expect("Failed to write to output file");
	} else {
		eprintln!("Usage: font_gen <input image> <output rust file>");
		std::process::exit(1);
	}
}
