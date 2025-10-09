//! ```cargo
//! [dependencies]
//! p8rs-types = { path = "../p8rs-types" }
//! ```

use p8rs_types::p8num::P8Num;

fn main() {
	println!("Hex,Sin,Cos,atan2-ne,atan2-nw,atan2-sw,atan2-se,Sqrt,x^2,2^x,ToDecimal");
	for id in 0 ..= 2_i32.pow(16) {
		let val = P8Num::from_raw(id);
		println!("{},{},{},{},{},{},{},{},{},{},{}",
			tostr(val),
			tostr(val.sin()),
			tostr(val.cos()),
			tostr(P8Num::atan2( P8Num::ONE - val, -val)),
			tostr(P8Num::atan2(-val,  val - P8Num::ONE)),
			tostr(P8Num::atan2( val - P8Num::ONE,  val)),
			tostr(P8Num::atan2( val,  P8Num::ONE - val)),
			tostr(val.powf(P8Num::new(0.5)).unwrap()),
			tostr(val.powf(P8Num::new(2.0)).unwrap()),
			tostr(P8Num::new(2.0).powf(val).unwrap()),
			val.to_str().as_ref(),
		)
	}
}

fn tostr(val: P8Num) -> String {
	let raw = val.to_raw() as u32;
	format!("0x{:04x}.{:04x}", raw >> 16, raw & 0xFFFF)
}
