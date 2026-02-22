use std::{env, process};
use std::fs::File;
use std::io::Write;
use getopts::Options;
use p8rs_types::p8num::P8Num;

#[cfg(windows)]
const EOL: &'static str = "\r\n";
#[cfg(not(windows))]
const EOL : &'static str = "\n";

fn main() {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflag("h", "help", "Print this help menu");
	
	let matches = opts.parse(&args[1..]).expect("Could not parse command line arguments");
	let help = matches.opt_present("h");
	let error = matches.free.len() != 1;
	
	if help || error {
		let brief = "Usage: numbers scripts/data/p8rs.csv [Options...]";
		if error {
			eprint!("{}", opts.usage(&brief));
			process::exit(-1);
		} else {
			print!("{}", opts.usage(&brief));
			return;
		}
	}
	
	let [output] = matches.free.try_into().unwrap();
	
	println!("Generating numbers -> {output}");
	
	let mut output = File::create(output).expect("Failed to create output file");
	
	write!(&mut output, "Hex,Sin,Cos,atan2-ne,atan2-nw,atan2-sw,atan2-se,Sqrt,x^2,2^x,ToDecimal{EOL}").expect("Failed to write to output file");
	for id in 0 ..= 2_i32.pow(16) {
		let val = P8Num::from_raw(id);
		write!(&mut output, "{},{},{},{},{},{},{},{},{},{},{}{EOL}",
		       tostr(val),
		       tostr(val.sin()),
		       tostr(val.cos()),
		       tostr(P8Num::atan2( P8Num::ONE - val, -val)),
		       tostr(P8Num::atan2(-val,  val - P8Num::ONE)),
		       tostr(P8Num::atan2( val - P8Num::ONE,  val)),
		       tostr(P8Num::atan2( val,  P8Num::ONE - val)),
		       tostr(val.powf(P8Num::new(0.5)).unwrap_or(P8Num::ZERO)),
		       tostr(val.powf(P8Num::new(2.0)).unwrap_or(P8Num::ZERO)),
		       tostr(P8Num::new(2.0).powf(val).unwrap_or(P8Num::ZERO)),
		       val.to_str().as_ref(),
		).expect("Failed to write to output file")
	}
}

fn tostr(val: P8Num) -> String {
	let raw = val.to_raw() as u32;
	format!("0x{:04x}.{:04x}", raw >> 16, raw & 0xFFFF)
}
