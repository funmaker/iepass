use alloc::vec::Vec;
use core::alloc::Allocator;
use anyhow::anyhow;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, RuntimeError, Value, Variadic};
use crate::pico8::memory::{MemoryDrawState, PrintAttributeFlags};
use crate::pico8::font::Font;
use crate::pico8::Runtime;

pub fn install_pico8_gfx<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("camera", camera::callback::<A>(ctx));
	ctx.set_global("color", color::callback::<A>(ctx));
	ctx.set_global("clip", clip::callback::<A>(ctx));
	ctx.set_global("pal", pal::callback::<A>(ctx));
	ctx.set_global("cls", cls::callback::<A>(ctx));
	ctx.set_global("cursor", cursor::callback::<A>(ctx));
}

pub fn draw_letter<A: Allocator>(_ctx: Context, rt: &mut Runtime<A>, flags: PrintAttributeFlags, letter: u8) -> Result<(i16, i16), RuntimeError> {
	// let is_wide = flags.contains(PrintAttributeFlags::WIDE);
	// let is_tall = flags.contains(PrintAttributeFlags::TALL);
	// let is_inverted = flags.contains(PrintAttributeFlags::INVERT);
	// let is_dotty = flags.contains(PrintAttributeFlags::DOTTY);
	let use_custom_font = flags.contains(PrintAttributeFlags::CUSTOM_FONT);
	
	let pen_color = *rt.memory.draw_state().pen_color();
	let [cursor_x, cursor_y] = *rt.memory.draw_state().cursor_position();
	
	let font = if use_custom_font { Font::new((&rt.memory[0x5600..=0x5dff]).try_into()?) } else { Font::SYSTEM };
	let char_width = font.width_chr(letter);
	let char_height = font.height();
	let char_font = &font.char(letter);
	
	assert!(char_width <= 8, "Char width cannot be >8");
	assert!(char_height <= 8, "Char height cannot be >8");
	
	for y in 0..char_height {
		let mut font_line = char_font[y as usize];
		for x in 0..char_width {
			let bit =  font_line & 1 != 0;
			font_line >>= 1;
			
			if bit {
				let pixel_x = cursor_x + x;
				let pixel_y = cursor_y + y;
				rt.memory.screen().set_pixel(pixel_x as i16, pixel_y as i16, pen_color).map_err(|_| anyhow!("drawing off screen"))?;
			}
		}
	}
	
	Ok((char_width as i16, char_height as i16))
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
pub fn cls<A: Allocator + 'static>(rt: &mut Runtime<A>, col: Option<i16>) -> Result<(), RuntimeError> {
	let col = col.unwrap_or(0) as u8 & 0xF;
	let col = (col << 4) | col;
	for byte in rt.memory.screen().iter_mut() {
		*byte = col;
	}
	*rt.memory.draw_state().cursor_position() = [0, 0];
	*rt.memory.draw_state().clip_rect() = [0, 0, 128, 128];
	Ok(())
}

#[api_callback]
pub fn cursor<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>, col: Option<i16>) -> Result<(u8, u8, u8), RuntimeError> {
	let prev_cursor = *rt.memory.draw_state().cursor_position();
	let prev_color = *rt.memory.draw_state().pen_color();
	
	set_cursor_color(&mut rt.memory.draw_state(), x.or(Some(0)), y.or(Some(0)), col);
	
	Ok((prev_cursor[0], prev_cursor[1], prev_color))
}

#[api_callback]
pub fn color<A: Allocator + 'static>(rt: &mut Runtime<A>, val: Option<u32>) -> Result<u8, RuntimeError> {
	let old = *rt.memory.draw_state().pen_color();
	if let Some(val) = val { *rt.memory.draw_state().pen_color() = val as u8; }
	Ok(old)
}

#[api_callback]
pub fn camera<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>) -> Result<(i16, i16), RuntimeError> {
	let old = (rt.memory.read(0x5f28), rt.memory.read(0x5f2a));
	rt.memory.write(0x5f28, x.unwrap_or(0));
	rt.memory.write(0x5f2a, y.unwrap_or(0));
	Ok(old)
}

#[api_callback]
pub fn clip<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<u8>, y: Option<u8>, w: Option<u8>, h: Option<u8>, clip_previous: Option<bool>) -> Result<(u8, u8, u8, u8), RuntimeError> {
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
	
	Ok((x_begin_old, y_begin_old, x_end_old, y_end_old))
}

#[api_callback]
pub fn pal<A: Allocator + 'static>(rt: &mut Runtime<A>, args: Variadic<Vec<Value>>) -> Result<(), RuntimeError> {
	let argc = args.len();
	assert!(argc >= 1 && argc <= 3, "Invalid number of arguments");
	
	if let Value::Table(new_pal) = args[0] {
		let pal_idx = if argc > 1 && let Some(p) = args[1].to_number() { p.to_integer() as u8 } else { 0 };
		let pal_base = rt.memory.base_addr_palette(pal_idx) as usize;
		for (idx, col) in new_pal {
			if let Some(k) = idx.to_number() && let Some(v) = col.to_number() {
				rt.memory[pal_base + k.to_integer().rem_euclid(16) as usize] = v.to_integer() as u8;
			}
		}
	} else if let Some(idx) = args[0].to_number() && let Some(col) = args[1].to_number() {
		let pal_idx = if argc > 2 && let Value::Number(p) = args[2] { p.to_integer() as u8 } else { 0 };
		let pal_base = rt.memory.base_addr_palette(pal_idx) as usize;
		rt.memory[pal_base + (idx.to_integer() % 16) as usize] = col.to_integer() as u8;
	} else {
		panic!("Invalid arguments");
	}
	
	Ok(())
}