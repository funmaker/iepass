use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::Context;
use p8rs_types::p8num::P8Num;
use crate::vm::Runtime;

pub fn install_pico8_drawing<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("rectfill", rectfill::callback::<A>(ctx));
}

#[api_callback]
pub fn rectfill<A: Allocator + 'static>(rt: &mut Runtime<A>, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	
	rt.memory.painter().paint_range(x0..=x1, y0..=y1);
}
