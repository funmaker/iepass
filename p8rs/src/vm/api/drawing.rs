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
	ctx.set_global("ovalfill", ovalfill::callback::<A>(ctx));
	ctx.set_global("oval", oval::callback::<A>(ctx));
	ctx.set_global("line", line::callback::<A>(ctx));
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

#[api_callback]
pub fn ovalfill<A: Allocator + 'static>(rt: &mut Runtime<A>, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	let mut painter = rt.memory.painter();
	
	let width = (x1 as i32 - x0 as i32) / 2;
	let height = (y1 as i32 - y0 as i32) / 2;
	let (ox0, oy0) = painter.to_abs(x0 + width as i16, y0 + height as i16);
	let (ox1, oy1) = painter.to_abs(x1 - width as i16, y1 - height as i16);
	
	if height == 0 {
		painter.paint_abs(ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy0);
		if oy0 != oy1 {
			painter.paint_abs(ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy1);
		}
		return;
	} else if width == 0 {
		painter.paint_abs(ox0, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
		if ox0 != ox1 {
			painter.paint_abs(ox1, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
		}
		return;
	}
	
	let a2 = width * width;
	let b2 = height * height;
	let crit1 = -(a2 / 4 + width % 2 + b2);
	let crit2 = -(b2 / 4 + height % 2 + a2);
	let crit3 = -(b2 / 4 + height % 2);
	let mut x = 0;
	let mut y = height;
	let mut t = -a2 * y;
	let mut dxt = 2 * b2 * x;
	let mut dyt = -2 * a2 * y;
	let d2xt = 2 * b2;
	let d2yt = 2 * a2;
	
	while y >= 0 && x <= width {
		if t + b2 * x <= crit1 || t + a2 * y <= crit3 {
			x += 1; dxt += d2xt; t += dxt;
		} else if t - a2 * y > crit2 {
			painter.paint_abs(ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy0.wrapping_sub(y as i16));
			painter.paint_abs(ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy1.wrapping_add(y as i16));
			y -= 1; dyt += d2yt; t += dyt;
		} else {
			painter.paint_abs(ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy0.wrapping_sub(y as i16));
			painter.paint_abs(ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy1.wrapping_add(y as i16));
			x += 1; dxt += d2xt; t += dxt;
			y -= 1; dyt += d2yt; t += dyt;
		}
	}
}

#[api_callback]
pub fn oval<A: Allocator + 'static>(rt: &mut Runtime<A>, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	let mut painter = rt.memory.painter();
	
	let width = (x1 as i32 - x0 as i32) / 2;
	let height = (y1 as i32 - y0 as i32) / 2;
	let (ox0, oy0) = painter.to_abs(x0 + width as i16, y0 + height as i16);
	let (ox1, oy1) = painter.to_abs(x1 - width as i16, y1 - height as i16);
	
	if height == 0 {
		painter.paint_abs(ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy0);
		if oy0 != oy1 {
			painter.paint_abs(ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy1);
		}
		return;
	} else if width == 0 {
		painter.paint_abs(ox0, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
		if ox0 != ox1 {
			painter.paint_abs(ox1, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
		}
		return;
	}
	
	let a2 = width * width;
	let b2 = height * height;
	let crit1 = -(a2 / 4 + width % 2 + b2);
	let crit2 = -(b2 / 4 + height % 2 + a2);
	let crit3 = -(b2 / 4 + height % 2);
	let mut x = 0;
	let mut y = height;
	let mut t = -a2 * y;
	let mut dxt = 2 * b2 * x;
	let mut dyt = -2 * a2 * y;
	let d2xt = 2 * b2;
	let d2yt = 2 * a2;
	
	while y >= 0 && x <= width {
		painter.paint_abs(ox1.wrapping_add(x as i16), oy1.wrapping_add(y as i16));
		painter.paint_abs(ox0.wrapping_sub(x as i16), oy0.wrapping_sub(y as i16));
		painter.paint_abs(ox1.wrapping_add(x as i16), oy0.wrapping_sub(y as i16));
		painter.paint_abs(ox0.wrapping_sub(x as i16), oy1.wrapping_add(y as i16));
		
		if t + b2 * x <= crit1 || t + a2 * y <= crit3 {
			x += 1; dxt += d2xt; t += dxt;
		} else if t - a2 * y > crit2 {
			y -= 1; dyt += d2yt; t += dyt;
		} else {
			x += 1; dxt += d2xt; t += dxt;
			y -= 1; dyt += d2yt; t += dyt;
		}
	}
}

#[api_callback]
pub fn line<A: Allocator + 'static>(rt: &mut Runtime<A>, p1: Option<P8Num>, p2: Option<P8Num>, p3: Option<P8Num>, p4: Option<P8Num>, p5: Option<P8Num>) {
	let (mut x0, mut y0, mut x1, mut y1) = match (p1, p2, p3, p4, p5) {
		(Some(x0), Some(y0), Some(x1), Some(y1), col) => {
			if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
			rt.memory.machine_state().set_line_endpoint(Some([x1.to_integer(), y1.to_integer()]));
			
			(x0.to_integer(), y0.to_integer(), x1.to_integer(), y1.to_integer())
		},
		(Some(x1), Some(y1), col, None, None) => {
			if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
			let prev = rt.memory.machine_state().get_line_endpoint();
			rt.memory.machine_state().set_line_endpoint(Some([x1.to_integer(), y1.to_integer()]));
			
			match prev {
				Some([x0, y0]) => (x0, y0, x1.to_integer(), y1.to_integer()),
				None => return,
			}
		},
		(Some(col), None, None, None, None) => {
			rt.memory.machine_state().set_line_endpoint(None);
			rt.memory.machine_state().set_pen_color(col);
			return;
		},
		(None, None, None, None, None) => {
			rt.memory.machine_state().set_line_endpoint(None);
			return;
		},
		_ => unreachable!(),
	};
	
	let mut painter = rt.memory.painter();
	
	if y0 == y1 {
		painter.paint(x0..=x1, y0);
		return;
	} else if x0 == x1 {
		painter.paint(x0, y0..=y1);
		return;
	}
	
	let steep = x1.abs_diff(x0) < y1.abs_diff(y0);
	if steep {
		(x0, y0) = (y0, x0);
		(x1, y1) = (y1, x1);
	}
	
	if x0 > x1 {
		(x0, x1) = (x1, x0);
		(y0, y1) = (y1, y0);
	}
	
	let dx = x1 - x0;
	let dy = y1 - y0;
	let derr = 2 * dy.abs();
	let mut err = 0;
	let mut y = y0;
	
	for x in x0..=x1 {
		if steep {
			painter.paint(y, x);
		} else {
			painter.paint(x, y);
		}
		
		err += derr;
		
		if err > dx {
			y += if y1 > y0 { 1 } else { -1 };
			err -= dx * 2;
		}
	}
}
