use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::Context;
use p8rs_types::p8num::P8Num;

use crate::vm::Runtime;

pub fn install_pico8_drawing<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("rectfill", rectfill::callback::<A>(ctx));
	ctx.set_global("rect", rect::callback::<A>(ctx));
	ctx.set_global("circfill", circfill::callback::<A>(ctx));
	ctx.set_global("circ", circ::callback::<A>(ctx));
}

#[api_callback]
pub fn rectfill<A: Allocator + 'static>(rt: &mut Runtime<A>, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	rt.memory
	  .painter()
	  .paint(x0..=x1, y0..=y1);
}

#[api_callback]
pub fn rect<A: Allocator + 'static>(rt: &mut Runtime<A>, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	rt.memory
	  .painter()
	  .paint(x0..=x1, y0)
	  .paint(x0..=x1, y1)
	  .paint(x0, y0..=y1)
	  .paint(x1, y0..=y1);
}

#[api_callback]
pub fn circfill<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, r: Option<i16>, col: Option<P8Num>) {
	let (Some(x), Some(y)) = (x, y) else { return };
	let r = r.unwrap_or(4);
	if r < 0 { return }
	
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	let painter = rt.memory.painter();
	let x0 = x.saturating_sub(r);
	let y0 = y.saturating_sub(r);
	let x1 = x.saturating_add(r);
	let y1 = y.saturating_add(r);
	let (x_mid, y_mid) = painter.to_abs(x, y);
	let r2 = (r as u32 * 2 + 1).pow(2) / 4;
	
	painter.with_callback(|x, y| (x as i32).abs_diff(x_mid as i32).pow(2) + (y as i32).abs_diff(y_mid as i32).pow(2) <= r2)
	       .paint(x0..=x1, y0..=y1);
}

#[api_callback]
pub fn circ<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, r: Option<i16>, col: Option<P8Num>) {
	let (Some(x), Some(y)) = (x, y) else { return };
	let r = r.unwrap_or(4);
	if r < 0 { return }
	
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	let painter = rt.memory.painter();
	let x0 = x.saturating_sub(r);
	let y0 = y.saturating_sub(r);
	let x1 = x.saturating_add(r);
	let y1 = y.saturating_add(r);
	let (x_mid, y_mid) = painter.to_abs(x, y);
	let r2 = (r as u32 * 2 + 1).pow(2) / 4;
	
	painter.with_callback(|x, y| (x as i32).abs_diff(x_mid as i32).pow(2) + (y as i32).abs_diff(y_mid as i32).pow(2) <= r2)
	       .paint(x0..=x1, y0..=y1);
}


