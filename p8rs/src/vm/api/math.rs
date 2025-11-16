use p8rs_macros::p8;
use p8rs_piccolo::Context;
use p8rs_macros::api_callback;

use crate::vm::P8Num;

pub fn install_pico8_math(ctx: Context) {
	ctx.set_global("abs", abs::callback(ctx));
	ctx.set_global("atan2", atan2::callback(ctx));
	ctx.set_global("ceil", ceil::callback(ctx));
	ctx.set_global("flr", flr::callback(ctx));
	ctx.set_global("min", min::callback(ctx));
	ctx.set_global("max", max::callback(ctx));
	ctx.set_global("mid", mid::callback(ctx));
	ctx.set_global("sgn", sgn::callback(ctx));
}

#[api_callback]
pub fn abs(v: P8Num) -> P8Num {
	v.abs()
}

#[api_callback]
pub fn atan2(dx: P8Num, dy: P8Num) -> P8Num {
	P8Num::atan2(dx, dy)
}

#[api_callback]
pub fn ceil(v: P8Num) -> P8Num {
	v.ceil()
}

#[api_callback]
pub fn flr(v: P8Num) -> P8Num {
	v.floor()
}

#[api_callback]
pub fn min(a: P8Num, b: P8Num) -> P8Num {
	a.min(b)
}

#[api_callback]
pub fn max(a: P8Num, b: P8Num) -> P8Num {
	a.max(b)
}

#[api_callback]
pub fn mid(a: P8Num, b: P8Num, c: P8Num) -> P8Num {
	if (a <= b) != (a <= c) { a } else if (b <= a) != (b <= c) { b } else { c }
}

#[api_callback]
pub fn sgn(v: P8Num) -> P8Num {
	if v < P8Num::ZERO { p8!(-1) } else { p8!(1) }
}
