use piccolo::Context;
use super::callback;

#[allow(unused_imports)]
use micromath::F32Ext;

pub fn install_pico8_math(ctx: Context) {
	
	ctx.set_global("abs", callback("abs", ctx, |_, v: f32| v.abs()));
	ctx.set_global("atan2", callback("atan2", ctx, |_, (dx, dy): (f32, f32)| dy.atan2(dx)));
	ctx.set_global("ceil", callback("ceil", ctx, |_, v: f32| v.ceil()));
	ctx.set_global("flr", callback("flr", ctx, |_, v: f32| v.floor()));
	ctx.set_global("min", callback("min", ctx, |_, (a, b): (f32, f32)| a.min(b)));
	ctx.set_global("max", callback("max", ctx, |_, (a, b): (f32, f32)| a.max(b)));
	ctx.set_global("mid", callback("mid", ctx, |_, (a, b, c): (f32, f32, f32)| if (a <= b) != (a <= c) { a } else if (b <= a) != (b <= c) { b } else { c }));
	ctx.set_global("sgn", callback("sgn", ctx, |_, v: f32| if v < 0f32 { -1 } else { 1 }));
	
}