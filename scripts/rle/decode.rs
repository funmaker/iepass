use std::fs::File;
use std::process;
use std::env;
use getopts::Options;
use p8rs::rle;

fn main() {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflag("h", "help", "Print this help menu");
	
	let matches = opts.parse(&args[1..]).expect("Could not parse command line arguments");
	let help = matches.opt_present("h");
	let error = matches.free.len() != 2;
	
	if help || error {
		let brief = "Usage: rle-decode <input file> <output file> [Options...]";
		if error {
			eprint!("{}", opts.usage(&brief));
			process::exit(-1);
		} else {
			print!("{}", opts.usage(&brief));
			return;
		}
	}
	
	let [input, output] = matches.free.try_into().unwrap();
	
	println!("RLE Decoding {input} -> {output}");
	std::io::copy(
		&mut rle::Decoder::new_std(&mut File::open(input).expect("Failed to open input file")),
		&mut File::create(output).expect("Failed to create output file"),
	).unwrap();
}
