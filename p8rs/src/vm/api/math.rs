use p8rs_macros::p8;
use p8rs_piccolo::Context;
use p8rs_macros::api_callback;

use crate::vm::P8Num;

pub fn load(ctx: Context) {
	ctx.set_global(b"band", band::callback(ctx));
	ctx.set_global(b"bnot", bnot::callback(ctx));
	ctx.set_global(b"bor", bor::callback(ctx));
	ctx.set_global(b"bxor", bxor::callback(ctx));
	ctx.set_global(b"ceil", ceil::callback(ctx));
	ctx.set_global(b"flr", flr::callback(ctx));
	ctx.set_global(b"abs", abs::callback(ctx));
	ctx.set_global(b"lshr", lshr::callback(ctx));
	ctx.set_global(b"max", max::callback(ctx));
	ctx.set_global(b"mid", mid::callback(ctx));
	ctx.set_global(b"min", min::callback(ctx));
	ctx.set_global(b"rotl", rotl::callback(ctx));
	ctx.set_global(b"rotr", rotr::callback(ctx));
	ctx.set_global(b"sgn", sgn::callback(ctx));
	ctx.set_global(b"shl", shl::callback(ctx));
	ctx.set_global(b"shr", shr::callback(ctx));
	ctx.set_global(b"sin", sin::callback(ctx));
	ctx.set_global(b"cos", cos::callback(ctx));
	ctx.set_global(b"atan2", atan2::callback(ctx));
	ctx.set_global(b"sqrt", sqrt::callback(ctx));
}

#[api_callback]
pub fn band(a: P8Num, b: P8Num) -> P8Num {
	a & b
}

#[api_callback]
pub fn bnot(a: P8Num) -> P8Num {
	!a
}

#[api_callback]
pub fn bor(a: P8Num, b: P8Num) -> P8Num {
	a | b
}

#[api_callback]
pub fn bxor(a: P8Num, b: P8Num) -> P8Num {
	a ^ b
}

#[api_callback]
pub fn ceil(a: P8Num) -> P8Num {
	a.ceil()
}

#[api_callback]
pub fn flr(a: P8Num) -> P8Num {
	a.floor()
}

#[api_callback]
pub fn abs(a: P8Num) -> P8Num {
	a.abs()
}

#[api_callback]
pub fn lshr(a: P8Num, b: P8Num) -> P8Num {
	if b < P8Num::ZERO {
		shl(a, -b)
	} else {
		a.to_raw()
		 .cast_unsigned()
		 .checked_shr(b.to_integer() as u32)
		 .map_or(P8Num::ZERO, |raw| P8Num::from_raw(raw as i32))
	}
}

#[api_callback]
pub fn max(a: P8Num, b: P8Num) -> P8Num {
	a.max(b)
}

#[api_callback]
pub fn mid(a: P8Num, b: P8Num, c: P8Num) -> P8Num {
	a.min(b).max(a.max(b).min(c))
}

#[api_callback]
pub fn min(a: P8Num, b: P8Num) -> P8Num {
	a.min(b)
}

#[api_callback]
pub fn rotl(a: P8Num, b: P8Num) -> P8Num {
	if b < P8Num::ZERO {
		rotr(a, -b)
	} else {
		P8Num::from_raw(
			a.to_raw()
			 .cast_unsigned()
			 .rotate_left(b.to_integer() as u32)
			 .cast_signed()
		)
	}
}

#[api_callback]
pub fn rotr(a: P8Num, b: P8Num) -> P8Num {
	if b < P8Num::ZERO {
		rotl(a, -b)
	} else {
		P8Num::from_raw(
			a.to_raw()
			 .cast_unsigned()
			 .rotate_right(b.to_integer() as u32)
			 .cast_signed()
		)
	}
}

#[api_callback]
pub fn sgn(v: P8Num) -> P8Num {
	if v < P8Num::ZERO { p8!(-1) } else { p8!(1) }
}

#[api_callback]
pub fn shl(a: P8Num, b: P8Num) -> P8Num {
	if b < P8Num::ZERO {
		shr(a, -b)
	} else {
		a.to_raw()
		 .checked_shl(b.to_integer() as u32)
		 .map_or(P8Num::ZERO, P8Num::from_raw)
	}
}

#[api_callback]
pub fn shr(a: P8Num, b: P8Num) -> P8Num {
	if b < P8Num::ZERO {
		shl(a, -b)
	} else {
		a.to_raw()
		 .checked_shr(b.to_integer() as u32)
		 .map_or(P8Num::ZERO, P8Num::from_raw)
	}
}

#[api_callback]
pub fn sin(a: P8Num) -> P8Num {
	a.sin()
}

#[api_callback]
pub fn cos(a: P8Num) -> P8Num {
	a.cos()
}

#[api_callback]
pub fn atan2(x: P8Num, y: P8Num) -> P8Num {
	P8Num::atan2(x, y)
}

#[api_callback]
pub fn sqrt(a: P8Num) -> P8Num {
	a.powf(p8!(0.5))
	 .unwrap_or(p8!(0))
}
