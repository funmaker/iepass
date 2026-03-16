use p8rs_macros::p8;
use p8rs_piccolo::Context;
use p8rs_types::p8num::P8Num;

pub const CONSTS: [(&[u8], P8Num); 26] = [
	(&p8!("█"), p8!("0x0000.8000"hex)),
	(&p8!("▒"), p8!("0x5a5a.8000"hex)),
	(&p8!("🐱"), p8!("0x511f.8000"hex)),
	(&p8!("░"), p8!("0x7d7d.8000"hex)),
	(&p8!("✽"), p8!("0xb81d.8000"hex)),
	(&p8!("●"), p8!("0xf99f.8000"hex)),
	(&p8!("♥"), p8!("0x51bf.8000"hex)),
	(&p8!("☉"), p8!("0xb5bf.8000"hex)),
	(&p8!("웃"), p8!("0x999f.8000"hex)),
	(&p8!("⌂"), p8!("0xb11f.8000"hex)),
	(&p8!("😐"), p8!("0xa0e0.8000"hex)),
	(&p8!("♪"), p8!("0x9b3f.8000"hex)),
	(&p8!("◆"), p8!("0xb1bf.8000"hex)),
	(&p8!("…"), p8!("0xf5ff.8000"hex)),
	(&p8!("★"), p8!("0xb15f.8000"hex)),
	(&p8!("⧗"), p8!("0x1b1f.8000"hex)),
	(&p8!("ˇ"), p8!("0xf5bf.8000"hex)),
	(&p8!("∧"), p8!("0x7adf.8000"hex)),
	(&p8!("▤"), p8!("0x0f0f.8000"hex)),
	(&p8!("▥"), p8!("0x5555.8000"hex)),
	(&p8!("⬅️"), p8!(0)),
	(&p8!("➡️"), p8!(1)),
	(&p8!("⬆️"), p8!(2)),
	(&p8!("⬇️"), p8!(3)),
	(&p8!("🅾️"), p8!(4)),
	(&p8!("❎"), p8!(5)),
];

pub fn load(ctx: Context) {
	for (key, val) in CONSTS {
		ctx.set_global(key, val);
	}
}
