use super::set_global_callback_simple;
use p8rs_piccolo::{Context, RuntimeError};

#[allow(unused_imports)]
use micromath::F32Ext;

pub fn install_pico8_math(ctx: Context) {
	set_global_callback_simple("abs", ctx, abs);
	set_global_callback_simple("atan2", ctx, atan2);
	set_global_callback_simple("ceil", ctx, ceil);
	set_global_callback_simple("flr", ctx, flr);
	set_global_callback_simple("min", ctx, min);
	set_global_callback_simple("max", ctx, max);
	set_global_callback_simple("mid", ctx, mid);
	set_global_callback_simple("sgn", ctx, sgn);
}

pub fn abs(v: f32) -> Result<f32, RuntimeError> {
	Ok(v.abs())
}
pub fn atan2((dx, dy): (f32, f32)) -> Result<f32, RuntimeError> {
	Ok(dy.atan2(dx))
}
pub fn ceil(v: f32) -> Result<f32, RuntimeError> { 
	Ok(v.ceil())
}
pub fn flr(v: f32) -> Result<f32, RuntimeError> {
	Ok(v.floor())
}
pub fn min((a, b): (f32, f32)) -> Result<f32, RuntimeError> {
	Ok(a.min(b))
}
pub fn max((a, b): (f32, f32)) -> Result<f32, RuntimeError> {
	Ok(a.max(b))
}
pub fn mid((a, b, c): (f32, f32, f32)) -> Result<f32, RuntimeError> { 
	Ok(if (a <= b) != (a <= c) { a } else if (b <= a) != (b <= c) { b } else { c })
}
pub fn sgn(v: f32) -> Result<i8, RuntimeError> {
	Ok(if v < 0f32 { -1 } else { 1 })
}