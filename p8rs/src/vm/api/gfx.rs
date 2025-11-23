use core::alloc::Allocator;
use p8rs_macros::{api_callback, p8};
use p8rs_piccolo::{Context, Value};
use p8rs_types::p8num::P8Num;

use crate::vm::memory::machine_state::{FillPatternFlags, FillPatternState, Palette};
use crate::vm::Runtime;

pub fn install_pico8_gfx<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("camera", camera::callback::<A>(ctx));
	ctx.set_global("color", color::callback::<A>(ctx));
	ctx.set_global("clip", clip::callback::<A>(ctx));
	ctx.set_global("pal", pal::callback::<A>(ctx));
	ctx.set_global("cls", cls::callback::<A>(ctx));
	ctx.set_global("cursor", cursor::callback::<A>(ctx));
	ctx.set_global("fillp", fillp::callback::<A>(ctx));
}

#[api_callback]
pub fn cls<A: Allocator + 'static>(rt: &mut Runtime<A>, col: Option<i16>) {
	let col = col.unwrap_or(0) as u8 & 0xF;
	let col = (col << 4) | col;
	
	rt.memory.screen().fill(col);
	rt.set_cursor_position([0, 0]);
	*rt.memory.machine_state().clip_rect() = [0, 0, 128, 128];
}

#[api_callback]
pub fn cursor<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, col: Option<P8Num>) -> (i16, i16, u8) {
	let [prev_x, prev_y] = rt.get_cursor_position();
	let prev_col = *rt.memory.machine_state().pen_color();
	let x = x.unwrap_or(0);
	let y = y.unwrap_or(0);
	
	rt.set_cursor_home(x);
	rt.set_cursor_position([x, y]);
	if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
	
	(prev_x, prev_y, prev_col)
}

#[api_callback]
pub fn color<A: Allocator + 'static>(rt: &mut Runtime<A>, col: Option<P8Num>) -> u8 {
	let prev = *rt.memory.machine_state().pen_color();
	
	rt.memory.machine_state().set_pen_color(col.unwrap_or(p8!(6)));
	
	prev
}

#[api_callback]
pub fn camera<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>) -> (i16, i16) {
	let [prev_x, prev_y] = rt.memory.machine_state().get_camera_position();
	
	rt.memory.machine_state().set_camera_x(x.unwrap_or(0));
	rt.memory.machine_state().set_camera_y(y.unwrap_or(0));
	
	(prev_x, prev_y)
}

#[api_callback]
pub fn clip<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, w: Option<i16>, h: Option<i16>, clip_previous: Option<bool>) -> (i16, i16, i16, i16) {
	let [prev_x_begin, prev_y_begin, prev_x_end, prev_y_end] = *rt.memory.machine_state().clip_rect();
	
	if let (Some(x), Some(y), Some(w), Some(h)) = (x, y, w, h) {
		let mut x_begin = x.clamp(0, 255) as u8;
		let mut y_begin = y.clamp(0, 255) as u8;
		let mut x_end = (x + w).clamp(0, 255) as u8;
		let mut y_end = (y + h).clamp(0, 255) as u8;
		
		if clip_previous.unwrap_or(false) {
			x_begin = x_begin.max(prev_x_begin);
			y_begin = y_begin.max(prev_y_begin);
			x_end = x_end.min(prev_x_end);
			y_end = y_end.min(prev_y_end);
		}
		
		*rt.memory.machine_state().clip_rect() = [x_begin, y_begin, x_end, y_end];
	} else {
		*rt.memory.machine_state().clip_rect() = [0, 0, 128, 128];
	}
	
	let prev_x = prev_x_begin as i16;
	let prev_y = prev_y_begin as i16;
	let prev_w = prev_x_end as i16 - prev_x;
	let prev_h = prev_y_end as i16 - prev_y;
	
	(prev_x, prev_y, prev_w, prev_h)
}

#[api_callback]
pub fn pal<'gc, A: Allocator + 'static>(rt: &mut Runtime<A>, c0: Option<Value<'gc>>, c1: Option<Value<'gc>>, p: Option<i16>) {
	if c0.is_none() {
		rt.memory.machine_state().reset_palette();
	} else if let Some(Value::Table(new_pal)) = c0 {
		let pal_idx = c1.and_then(|val| val.to_number())
		                .map(|val| val.to_integer())
		                .unwrap_or(0);
		
		if let Some(pal) = Palette::new(pal_idx) {
			let mut ds = rt.memory.machine_state();
			let pal = ds.palette(pal);
			
			for (idx, col) in new_pal.iter() {
				if let (Some(k), Some(v)) = (idx.to_number(), col.to_number()) {
					let k = k.to_integer().cast_unsigned() as usize % 16;
					pal[k] = v.to_integer() as u8;
				}
			}
		}
	} else if let (Some(k), Some(v), Some(pal)) = (c0.and_then(Value::to_number), c1.and_then(Value::to_number), Palette::new(p.unwrap_or(0))) {
		let mut ds = rt.memory.machine_state();
		let pal = ds.palette(pal);
		let k = k.to_integer().cast_unsigned() as usize % 16;
		pal[k] = v.to_integer() as u8;
	}
}

#[api_callback]
pub fn fillp<'gc, A: Allocator + 'static>(rt: &mut Runtime<A>, pat: Option<P8Num>) {
	let pat = pat.unwrap_or(P8Num::ZERO);
	let flags = (pat.to_raw() >> 8) as u8;
	let flags = flags.reverse_bits() & 0b0000_0111;
	let flags = FillPatternFlags::from_bits_retain(flags);
	let pattern = pat.to_integer().cast_unsigned();
	
	*rt.memory.machine_state().fill_pattern() = FillPatternState::new(pattern, flags);
}