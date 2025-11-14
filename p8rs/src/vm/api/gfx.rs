use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Value};

use crate::vm::memory::{MemoryDrawState, Palette};
use crate::vm::Runtime;

pub fn install_pico8_gfx<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("camera", camera::callback::<A>(ctx));
	ctx.set_global("color", color::callback::<A>(ctx));
	ctx.set_global("clip", clip::callback::<A>(ctx));
	ctx.set_global("pal", pal::callback::<A>(ctx));
	ctx.set_global("cls", cls::callback::<A>(ctx));
	ctx.set_global("cursor", cursor::callback::<A>(ctx));
}

pub fn set_cursor_color<A: Allocator>(draw_state: &mut MemoryDrawState<A>, x: Option<i16>, y: Option<i16>, color: Option<i16>) {
	if let Some(x) = x {
		*draw_state.cursor_home_x() = x as u8;
		draw_state.cursor_position()[0] = x as u8;
	}
	
	if let Some(y) = y {
		draw_state.cursor_position()[1] = y as u8;
	}
	
	if let Some(color) = color {
		*draw_state.pen_color() = color as u8;
	}
}

#[api_callback]
pub fn cls<A: Allocator + 'static>(rt: &mut Runtime<A>, col: Option<i16>) {
	let col = col.unwrap_or(0) as u8 & 0xF;
	let col = (col << 4) | col;
	rt.memory.screen().fill(col);
	*rt.memory.draw_state().cursor_position() = [0, 0];
	*rt.memory.draw_state().clip_rect() = [0, 0, 128, 128];
}

#[api_callback]
pub fn cursor<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, col: Option<i16>) -> (u8, u8, u8) {
	let prev_cursor = *rt.memory.draw_state().cursor_position();
	let prev_color = *rt.memory.draw_state().pen_color();
	
	set_cursor_color(&mut rt.memory.draw_state(), x.or(Some(0)), y.or(Some(0)), col);
	
	(prev_cursor[0], prev_cursor[1], prev_color)
}

#[api_callback]
pub fn color<A: Allocator + 'static>(rt: &mut Runtime<A>, val: Option<u8>) -> u8 {
	let old = *rt.memory.draw_state().pen_color();
	if let Some(val) = val {
		*rt.memory.draw_state().pen_color() = val;
	}
	old
}

#[api_callback]
pub fn camera<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>) -> (i16, i16) {
	let old = rt.memory.draw_state().get_camera_position();
	rt.memory.draw_state().set_camera_x(x.unwrap_or(0));
	rt.memory.draw_state().set_camera_y(y.unwrap_or(0));
	(old[0], old[1])
}

#[api_callback]
pub fn clip<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<u8>, y: Option<u8>, w: Option<u8>, h: Option<u8>, clip_previous: Option<bool>) -> (u8, u8, u8, u8) {
	let [x_begin_old, y_begin_old, x_end_old, y_end_old] = *rt.memory.draw_state().clip_rect();
	
	if let Some(x) = x && let Some(y) = y && let Some(w) = w && let Some(h) = h {
		let mut x_begin = x;
		let mut y_begin = y;
		let mut x_end = x + w;
		let mut y_end = y + h;
		
		if clip_previous.unwrap_or(false) {
			if x_begin < x_begin_old { x_begin = x_begin_old; }
			if y_begin < y_begin_old { y_begin = y_begin_old; }
			if x_end > x_end_old { x_end = x_end_old; }
			if y_end > y_end_old { y_end = y_end_old; }
		}
		
		*rt.memory.draw_state().clip_rect() = [x_begin, y_begin, x_end.min(128), y_end.min(128)];
	} else {
		*rt.memory.draw_state().clip_rect() = [0, 0, 128, 128];
	}
	
	(x_begin_old, y_begin_old, x_end_old, y_end_old)
}

#[api_callback]
pub fn pal<'gc, A: Allocator + 'static>(rt: &mut Runtime<A>, c0: Option<Value<'gc>>, c1: Option<Value<'gc>>, p: Option<i16>) {
	if c0.is_none() {
		rt.memory.draw_state().reset_palette();
	} else if let Some(Value::Table(new_pal)) = c0 {
		let pal_idx = c1.and_then(|val| val.to_number())
		                .map(|val| val.to_integer())
		                .unwrap_or(0);
		
		if let Some(pal) = Palette::new(pal_idx) {
			let mut ds = rt.memory.draw_state();
			let pal = ds.palette(pal);
			
			for (idx, col) in new_pal.iter() {
				if let (Some(k), Some(v)) = (idx.to_number(), col.to_number()) {
					let k = k.to_integer().cast_unsigned() as usize % 16;
					pal[k] = v.to_integer() as u8;
				}
			}
		}
	} else if let (Some(k), Some(v), Some(pal)) = (c0.and_then(Value::to_number), c1.and_then(Value::to_number), Palette::new(p.unwrap_or(0))) {
		let mut ds = rt.memory.draw_state();
		let pal = ds.palette(pal);
		let k = k.to_integer().cast_unsigned() as usize % 16;
		pal[k] = v.to_integer() as u8;
	}
}