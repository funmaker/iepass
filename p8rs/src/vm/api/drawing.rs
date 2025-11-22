// Circle algorithms taken from https://github.com/egordorichev/pemsa/blob/master/src/pemsa/graphics/pemsa_graphics_api.cpp

use core::alloc::Allocator;
use p8rs_macros::{api_callback, p8};
use p8rs_piccolo::Context;
use p8rs_types::p8num::P8Num;

use crate::vm::memory::machine_state::MiscChipsetFeatureFlags;
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
pub fn circfill<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, r: Option<P8Num>, col: Option<P8Num>) {
	let (Some(x), Some(y)) = (x, y) else { return };
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	let r = r.unwrap_or(p8!(4));
	if r < p8!(0) { return }
	
	let even_flag = rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::EVEN_RADIUS_CIRC);
	let mut painter = rt.memory.painter();
	
	let (x0, y0) = painter.to_abs(x, y);
	let (x1, y1) = if r.fract() >= p8!(0.5) && even_flag {
		(x0.wrapping_add(1), y0.wrapping_add(1))
	} else {
		(x0, y0)
	};
	
	let mut x = r.to_integer();
	let mut y = 0;
	let mut e = 1 - x;
	while x >= y {
		painter.paint_abs(x0.wrapping_sub(x)..=x1.wrapping_add(x), y0.wrapping_sub(y));
		painter.paint_abs(x0.wrapping_sub(x)..=x1.wrapping_add(x), y1.wrapping_add(y));
		
		if e < 0 {
			y += 1;
			e += 2 * y + 1;
		} else {
			if x != y {
				painter.paint_abs(x0.wrapping_sub(y)..=x1.wrapping_add(y), y0.wrapping_sub(x));
				painter.paint_abs(x0.wrapping_sub(y)..=x1.wrapping_add(y), y1.wrapping_add(x));
			}
			
			y += 1;
			x -= 1;
			e += 2 * (y - x) + 1;
		}
	}
}


#[api_callback]
pub fn circ<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, r: Option<P8Num>, col: Option<P8Num>) {
	let (Some(x), Some(y)) = (x, y) else { return };
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	let r = r.unwrap_or(p8!(4));
	if r < p8!(0) { return }
	
	let even_flag = rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::EVEN_RADIUS_CIRC);
	let mut painter = rt.memory.painter();
	
	let (x0, y0) = painter.to_abs(x, y);
	let (x1, y1) = if r.fract() >= p8!(0.5) && even_flag {
		(x0.wrapping_add(1), y0.wrapping_add(1))
	} else {
		(x0, y0)
	};
	
	let mut x = r.to_integer();
	let mut y = 0;
	let mut e = 1 - x;
	while x >= y {
		painter.paint_abs(x1.wrapping_add(x), y1.wrapping_add(y));
		painter.paint_abs(x1.wrapping_add(y), y1.wrapping_add(x));
		painter.paint_abs(x1.wrapping_add(x), y0.wrapping_sub(y));
		painter.paint_abs(x1.wrapping_add(y), y0.wrapping_sub(x));
		painter.paint_abs(x0.wrapping_sub(x), y1.wrapping_add(y));
		painter.paint_abs(x0.wrapping_sub(y), y1.wrapping_add(x));
		painter.paint_abs(x0.wrapping_sub(x), y0.wrapping_sub(y));
		painter.paint_abs(x0.wrapping_sub(y), y0.wrapping_sub(x));
		
		y += 1;
		if e < 0 {
			e += 2 * y + 1;
		} else {
			x -= 1;
			e += 2 * (y - x) + 1;
		}
	}
}


