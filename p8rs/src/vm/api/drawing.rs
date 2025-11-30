// Circle algorithms taken from https://github.com/egordorichev/pemsa/blob/master/src/pemsa/graphics/pemsa_graphics_api.cpp

use core::alloc::Allocator;
use p8rs_macros::{api_callback, p8};
use p8rs_piccolo::Context;
use p8rs_types::p8num::P8Num;

use crate::vm::memory::machine_state::MiscChipsetFeatureFlags;
use crate::vm::memory::Memory;
use crate::vm::Runtime;

pub fn install_pico8_drawing<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("rectfill", rectfill::callback::<A>(ctx));
	ctx.set_global("rect", rect::callback::<A>(ctx));
	ctx.set_global("circfill", circfill::callback::<A>(ctx));
	ctx.set_global("circ", circ::callback::<A>(ctx));
	ctx.set_global("ovalfill", ovalfill::callback::<A>(ctx));
	ctx.set_global("oval", oval::callback::<A>(ctx));
	ctx.set_global("line", line::callback::<A>(ctx));
	ctx.set_global("spr", spr::callback::<A>(ctx));
	ctx.set_global("sspr", sspr::callback::<A>(ctx));
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

#[api_callback]
pub fn spr<A: Allocator + 'static>(rt: &mut Runtime<A>, n: Option<i16>, x: Option<i16>, y: Option<i16>, w: Option<P8Num>, h: Option<P8Num>, flip_x: Option<bool>, flip_y: Option<bool>) {
	let n = n.unwrap_or(0);
	let sx = (n % 16) * 8;
	let sy = (n / 16) * 8;
	let x = x.unwrap_or(0);
	let y = y.unwrap_or(0);
	let w = (w.unwrap_or(p8!(1)) * p8!(8)).to_integer().clamp(0, 128 - sx);
	let h = (h.unwrap_or(p8!(1)) * p8!(8)).to_integer().clamp(0, 128 - sy);
	let flip_x = flip_x.unwrap_or(false);
	let flip_y = flip_y.unwrap_or(false);
	if n < 0 || n > 255 || w <= 0 || h <= 0 { return; }
	
	let painter = rt.memory.painter().sprite_mode();
	let (x0, y0) = painter.to_abs(x, y);
	let x1 = x0 + w - 1;
	let y1 = y0 + h - 1;
	
	match (flip_x, flip_y) {
		(false, false) => { painter.with_callback(|memory: &mut Memory, x, y| memory.sprites().get_pixel((sx + x as i16 - x0) as u8, (sy + y as i16 - y0) as u8)).paint_abs(x0..=x1, y0..=y1); },
		(true,  false) => { painter.with_callback(|memory: &mut Memory, x, y| memory.sprites().get_pixel((sx + x1 - x as i16) as u8, (sy + y as i16 - y0) as u8)).paint_abs(x0..=x1, y0..=y1); },
		(false, true ) => { painter.with_callback(|memory: &mut Memory, x, y| memory.sprites().get_pixel((sx + x as i16 - x0) as u8, (sy + y1 - y as i16) as u8)).paint_abs(x0..=x1, y0..=y1); },
		(true,  true ) => { painter.with_callback(|memory: &mut Memory, x, y| memory.sprites().get_pixel((sx + x1 - x as i16) as u8, (sy + y1 - y as i16) as u8)).paint_abs(x0..=x1, y0..=y1); },
	}
}

#[api_callback]
pub fn sspr<A: Allocator + 'static>(rt: &mut Runtime<A>, sx: i16, sy: i16, sw: i16, sh: i16, mut dx: i16, mut dy: i16, dw: Option<i16>, dh: Option<i16>, flip_x: Option<bool>, flip_y: Option<bool>) {
	let mut dw = dw.unwrap_or(sw);
	let mut dh = dh.unwrap_or(sw);
	let mut flip_x = flip_x.unwrap_or(false);
	let mut flip_y = flip_y.unwrap_or(false);
	if sw <= 0 || sh <= 0 || dw == 0 || dh == 0 { return; }
	
	if dw <= 0 {
		dx += dw;
		dw = -dw;
		flip_x = !flip_x;
	}
	
	if dh <= 0 {
		dy += dh;
		dh = -dh;
		flip_y = !flip_y;
	}
	
	let painter = rt.memory.painter().sprite_mode();
	let (dx0, dy0) = painter.to_abs(dx, dy);
	let sx0 = sx;
	let sy0 = sy;
	let sx1 = sx + sw - 1;
	let sy1 = sy + sh - 1;
	let dxf = (sw > dw && sw % dw == 0).then_some(sw / dw);
	let dyf = (sh > dh && sh % dh == 0).then_some(sh / dh);
	
	painter
		.with_callback(|memory: &mut Memory, x, y| {
			let mut sx = sx0 + sample(x as i16 - dx0, dw, sw, dxf);
			let mut sy = sy0 + sample(y as i16 - dy0, dh, sh, dyf);
			
			if flip_x {
				sx = sx1.min(127) - (sx - sx0.max(0))
			}
			
			if flip_y {
				sy = sy1.min(127) - (sy - sy0.max(0))
			}
			
			if sx < sx0.max(0) || sy < sy0.max(0) || sx > sx1.min(127) || sy > sy1.min(127) {
				return None
			}
			
			let sx = u8::try_from(sx).ok()?;
			let sy = u8::try_from(sy).ok()?;
			
			memory.sprites().get_pixel(sx, sy)
		})
		.paint_abs(dx0..dx0+dw, dy0..dy0+dh);
}

fn sample(x: i16, dw: i16, sw: i16, factor: Option<i16>) -> i16 {
	let ret = if let Some(factor) = factor {
		x * factor + factor / 2
	} else {
		((2 * x + 1) * sw - 1) / (2 * dw)
	};
	
	ret
}
