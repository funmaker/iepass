// Circle algorithms taken from https://github.com/egordorichev/pemsa/blob/master/src/pemsa/graphics/pemsa_graphics_api.cpp

use p8rs_macros::{api_callback, p8};
use p8rs_piccolo::Context;
use p8rs_types::p8num::P8Num;

use crate::vm::memory::machine_state::MiscChipsetFeatureFlags;
use crate::vm::memory::Memory;
use crate::vm::Runtime;

pub fn load(ctx: Context) {
	ctx.set_global(b"rectfill", rectfill::callback(ctx));
	ctx.set_global(b"rect", rect::callback(ctx));
	ctx.set_global(b"circfill", circfill::callback(ctx));
	ctx.set_global(b"circ", circ::callback(ctx));
	ctx.set_global(b"ovalfill", ovalfill::callback(ctx));
	ctx.set_global(b"oval", oval::callback(ctx));
	ctx.set_global(b"line", line::callback(ctx));
	ctx.set_global(b"spr", spr::callback(ctx));
	ctx.set_global(b"sspr", sspr::callback(ctx));
	ctx.set_global(b"map", map::callback(ctx));
	ctx.set_global(b"mapdraw", map::callback(ctx));
}

#[api_callback]
pub fn rectfill(rt: &mut Runtime, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	rt.memory
	  .painter()
	  .paint(&mut rt.memory, x0..=x1, y0..=y1);
}

#[api_callback]
pub fn rect(rt: &mut Runtime, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	rt.memory
	  .painter()
	  .paint(&mut rt.memory, x0..=x1, y0)
	  .paint(&mut rt.memory, x0..=x1, y1)
	  .paint(&mut rt.memory, x0, y0..=y1)
	  .paint(&mut rt.memory, x1, y0..=y1);
}

#[api_callback]
pub fn circfill(rt: &mut Runtime, x: Option<i16>, y: Option<i16>, r: Option<P8Num>, col: Option<P8Num>) {
	let (Some(x), Some(y)) = (x, y) else { return };
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	let r = r.unwrap_or(p8!(4));
	if r < p8!(0) { return }
	
	let even_flag = rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::EVEN_RADIUS_CIRC);
	let painter = rt.memory.painter();
	
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
		painter.paint_abs(&mut rt.memory, x0.wrapping_sub(x)..=x1.wrapping_add(x), y0.wrapping_sub(y));
		painter.paint_abs(&mut rt.memory, x0.wrapping_sub(x)..=x1.wrapping_add(x), y1.wrapping_add(y));
		
		if e < 0 {
			y += 1;
			e += 2 * y + 1;
		} else {
			if x != y {
				painter.paint_abs(&mut rt.memory, x0.wrapping_sub(y)..=x1.wrapping_add(y), y0.wrapping_sub(x));
				painter.paint_abs(&mut rt.memory, x0.wrapping_sub(y)..=x1.wrapping_add(y), y1.wrapping_add(x));
			}
			
			y += 1;
			x -= 1;
			e += 2 * (y - x) + 1;
		}
	}
}


#[api_callback]
pub fn circ(rt: &mut Runtime, x: Option<i16>, y: Option<i16>, r: Option<P8Num>, col: Option<P8Num>) {
	let (Some(x), Some(y)) = (x, y) else { return };
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	let r = r.unwrap_or(p8!(4));
	if r < p8!(0) { return }
	
	let even_flag = rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::EVEN_RADIUS_CIRC);
	let painter = rt.memory.painter();
	
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
		painter.paint_abs(&mut rt.memory, x1.wrapping_add(x), y1.wrapping_add(y));
		painter.paint_abs(&mut rt.memory, x1.wrapping_add(y), y1.wrapping_add(x));
		painter.paint_abs(&mut rt.memory, x1.wrapping_add(x), y0.wrapping_sub(y));
		painter.paint_abs(&mut rt.memory, x1.wrapping_add(y), y0.wrapping_sub(x));
		painter.paint_abs(&mut rt.memory, x0.wrapping_sub(x), y1.wrapping_add(y));
		painter.paint_abs(&mut rt.memory, x0.wrapping_sub(y), y1.wrapping_add(x));
		painter.paint_abs(&mut rt.memory, x0.wrapping_sub(x), y0.wrapping_sub(y));
		painter.paint_abs(&mut rt.memory, x0.wrapping_sub(y), y0.wrapping_sub(x));
		
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
pub fn ovalfill(rt: &mut Runtime, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	let painter = rt.memory.painter();
	
	let width = (x1 as i32 - x0 as i32) / 2;
	let height = (y1 as i32 - y0 as i32) / 2;
	let (ox0, oy0) = painter.to_abs(x0 + width as i16, y0 + height as i16);
	let (ox1, oy1) = painter.to_abs(x1 - width as i16, y1 - height as i16);
	
	if height == 0 {
		painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy0);
		if oy0 != oy1 {
			painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy1);
		}
		return;
	} else if width == 0 {
		painter.paint_abs(&mut rt.memory, ox0, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
		if ox0 != ox1 {
			painter.paint_abs(&mut rt.memory, ox1, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
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
			painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy0.wrapping_sub(y as i16));
			painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy1.wrapping_add(y as i16));
			y -= 1; dyt += d2yt; t += dyt;
		} else {
			painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy0.wrapping_sub(y as i16));
			painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(x as i16)..=ox1.wrapping_add(x as i16), oy1.wrapping_add(y as i16));
			x += 1; dxt += d2xt; t += dxt;
			y -= 1; dyt += d2yt; t += dyt;
		}
	}
}

#[api_callback]
pub fn oval(rt: &mut Runtime, x0: Option<i16>, y0: Option<i16>, x1: Option<i16>, y1: Option<i16>, col: Option<P8Num>) {
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	let x0 = x0.unwrap_or(0);
	let y0 = y0.unwrap_or(0);
	let x1 = x1.unwrap_or(0);
	let y1 = y1.unwrap_or(0);
	let (x0, x1) = (x0.min(x1), x0.max(x1));
	let (y0, y1) = (y0.min(y1), y0.max(y1));
	
	let painter = rt.memory.painter();
	
	let width = (x1 as i32 - x0 as i32) / 2;
	let height = (y1 as i32 - y0 as i32) / 2;
	let (ox0, oy0) = painter.to_abs(x0 + width as i16, y0 + height as i16);
	let (ox1, oy1) = painter.to_abs(x1 - width as i16, y1 - height as i16);
	
	if height == 0 {
		painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy0);
		if oy0 != oy1 {
			painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(width as i16)..=ox1.wrapping_add(width as i16), oy1);
		}
		return;
	} else if width == 0 {
		painter.paint_abs(&mut rt.memory, ox0, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
		if ox0 != ox1 {
			painter.paint_abs(&mut rt.memory, ox1, oy0.wrapping_sub(height as i16)..=oy1.wrapping_add(height as i16));
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
		painter.paint_abs(&mut rt.memory, ox1.wrapping_add(x as i16), oy1.wrapping_add(y as i16));
		painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(x as i16), oy0.wrapping_sub(y as i16));
		painter.paint_abs(&mut rt.memory, ox1.wrapping_add(x as i16), oy0.wrapping_sub(y as i16));
		painter.paint_abs(&mut rt.memory, ox0.wrapping_sub(x as i16), oy1.wrapping_add(y as i16));
		
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
pub fn line(rt: &mut Runtime, p1: Option<P8Num>, p2: Option<P8Num>, p3: Option<P8Num>, p4: Option<P8Num>, p5: Option<P8Num>) {
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
	
	let painter = rt.memory.painter();
	
	if y0 == y1 {
		painter.paint(&mut rt.memory, x0..=x1, y0);
		return;
	} else if x0 == x1 {
		painter.paint(&mut rt.memory, x0, y0..=y1);
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
			painter.paint(&mut rt.memory, y, x);
		} else {
			painter.paint(&mut rt.memory, x, y);
		}
		
		err += derr;
		
		if err > dx {
			y += if y1 > y0 { 1 } else { -1 };
			err -= dx * 2;
		}
	}
}

#[api_callback]
pub fn spr(rt: &mut Runtime, n: Option<i16>, x: Option<i16>, y: Option<i16>, w: Option<P8Num>, h: Option<P8Num>, flip_x: Option<bool>, flip_y: Option<bool>) {
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
	
	let memory = &mut rt.memory;
	let graphics = memory.graphics();
	let sprites = graphics.sprites();
	let painter = graphics.painter(memory).sprite_mode(memory);
	let (x0, y0) = painter.to_abs(x, y);
	let x1 = x0 + w - 1;
	let y1 = y0 + h - 1;
	
	match (flip_x, flip_y) {
		(false, false) => { painter.paint_abs_tex(memory, x0..=x1, y0..=y1, |memory: &mut Memory, x, y| sprites.get_pixel(memory, (sx + x as i16 - x0) as u8, (sy + y as i16 - y0) as u8)); },
		(true,  false) => { painter.paint_abs_tex(memory, x0..=x1, y0..=y1, |memory: &mut Memory, x, y| sprites.get_pixel(memory, (sx + x1 - x as i16) as u8, (sy + y as i16 - y0) as u8)); },
		(false, true ) => { painter.paint_abs_tex(memory, x0..=x1, y0..=y1, |memory: &mut Memory, x, y| sprites.get_pixel(memory, (sx + x as i16 - x0) as u8, (sy + y1 - y as i16) as u8)); },
		(true,  true ) => { painter.paint_abs_tex(memory, x0..=x1, y0..=y1, |memory: &mut Memory, x, y| sprites.get_pixel(memory, (sx + x1 - x as i16) as u8, (sy + y1 - y as i16) as u8)); },
	}
}

#[api_callback]
pub fn sspr(rt: &mut Runtime, sx: i16, sy: i16, sw: i16, sh: i16, mut dx: i16, mut dy: i16, dw: Option<i16>, dh: Option<i16>, flip_x: Option<bool>, flip_y: Option<bool>) {
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
	
	let memory = &mut rt.memory;
	let graphics = memory.graphics();
	let painter = graphics.painter(memory).sprite_mode(memory);
	let sprites = graphics.sprites();
	let (dx0, dy0) = painter.to_abs(dx, dy);
	let sx0 = sx;
	let sy0 = sy;
	let sx1 = sx + sw - 1;
	let sy1 = sy + sh - 1;
	let dxf = (sw > dw && sw % dw == 0).then_some(sw / dw);
	let dyf = (sh > dh && sh % dh == 0).then_some(sh / dh);
	
	painter.paint_abs_tex(&mut rt.memory, dx0..dx0+dw, dy0..dy0+dh, |memory: &mut Memory, x, y| {
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
		
		sprites.get_pixel(memory, sx, sy)
	});
}

fn sample(x: i16, dw: i16, sw: i16, factor: Option<i16>) -> i16 {
	let ret = if let Some(factor) = factor {
		x * factor + factor / 2
	} else {
		((2 * x + 1) * sw - 1) / (2 * dw)
	};
	
	ret
}

#[api_callback]
pub fn map(rt: &mut Runtime, mx: Option<i16>, my: Option<i16>, dx: Option<i16>, dy: Option<i16>, mw: Option<i16>, mh: Option<i16>, layer: Option<i16>) {
	let memory = &mut rt.memory;
	let graphics = memory.graphics();
	let sprites = graphics.sprites();
	let map = graphics.map(memory);
	let painter = graphics.painter(memory).sprite_mode(memory);
	
	let mx = mx.unwrap_or(0);
	let my = my.unwrap_or(0);
	let dx = dx.unwrap_or(0);
	let dy = dy.unwrap_or(0);
	let mut mw = mw.map(|sw| sw.max(0) as u16).unwrap_or(map.width());
	let mut mh = mh.map(|sh| sh.max(0) as u16).unwrap_or(map.height().min(0x7FFF));
	
	let layer = match layer {
		None | Some(0) => 0xFF,
		Some(val) => val as u8,
	};
	
	let (mut x0, mut y0) = painter.to_abs(dx, dy);
	
	let mut mx = if mx < 0 {
		x0 = x0.saturating_add(mx.saturating_neg().saturating_mul(8));
		mw = mw.saturating_sub(mx.saturating_neg() as u16);
		0
	} else {
		mx as u16
	};
	
	let mut my = if my < 0 {
		y0 = y0.saturating_add(my.saturating_neg().saturating_mul(8));
		mh = mh.saturating_sub(my.saturating_neg() as u16);
		0
	} else {
		my as u16
	};
	
	if x0 <= -8 {
		let cells = (x0 / -8) as u16;
		mx = mx.saturating_add(cells);
		mw = mw.saturating_sub(cells);
		x0 %= 8;
	}
	
	if y0 <= -8 {
		let cells = (y0 / -8) as u16;
		my = my.saturating_add(cells);
		mh = mh.saturating_sub(cells);
		y0 %= 8;
	}
	
	if mx + mw > map.width() {
		mw = map.width().saturating_sub(mx);
	}
	
	if my + mh > map.height() {
		mh = map.height().saturating_sub(my);
	}
	
	if mw <= 0 || mh <= 0 || x0 >= 128 || y0 >= 128 || mx >= map.width() || my >= map.height() || layer == 0 { return; }
	
	let mw = mw.min(17);
	let mh = mh.min(17);
	
	for cy in 0..mh {
		for cx in 0..mw {
			let sprite = map.get_sprite(memory, mx + cx, my + cy).unwrap();
			
			let [sx, sy] = sprites.sprite_pos(sprite).map(i16::from);
			let sx0 = x0 + cx as i16 * 8;
			let sy0 = y0 + cy as i16 * 8;
			
			if sprite == 0 || layer & memory.sprite_flags()[sprite] == 0 { continue }
			
			painter.paint_abs_tex(
				memory,
				sx0..sx0+8,
				sy0..sy0+8,
				|memory: &mut Memory, x, y| sprites.get_pixel(memory, (sx + x as i16 - sx0) as u8, (sy + y as i16 - sy0) as u8),
			);
		}
	}
}
