use super::{set_global_callback_env, EnvHandle};
use alloc::vec::Vec;
use core::alloc::Allocator;
use p8rs_piccolo::{Context, RuntimeError, Value, Variadic};
use crate::pico8::font::Font;
use crate::pico8::memory::{MemoryDrawState, PrintAttributeFlags};

pub fn install_pico8_gfx<A: Allocator + Clone + 'static>(env_orig: EnvHandle<A>, ctx: Context) {
	set_global_callback_env("cls", ctx, env_orig.clone(), cls);
	set_global_callback_env("cursor", ctx, env_orig.clone(), cursor);
	set_global_callback_env("camera", ctx, env_orig.clone(), camera);
	set_global_callback_env("color", ctx, env_orig.clone(), color);
	set_global_callback_env("clip", ctx, env_orig.clone(), clip);
	set_global_callback_env("pal", ctx, env_orig.clone(), pal);
	// set_global_callback_env("print", ctx, env_orig.clone(), print);
}

fn set_cursor_color<A: Allocator + Clone>(draw_state: &mut MemoryDrawState<A>, x: Option<i16>, y: Option<i16>, color: Option<i16>) {
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

// todo: wip
// pub fn draw_letter<A: Allocator + Clone + 'static>(env: EnvHandle<A>, flags: PrintAttributeFlags, letter: u8) {
// 	let is_wide = flags.contains(PrintAttributeFlags::WIDE);
// 	let is_tall = flags.contains(PrintAttributeFlags::TALL);
// 	let is_inverted = flags.contains(PrintAttributeFlags::INVERT);
// 	let is_dotty = flags.contains(PrintAttributeFlags::DOTTY);
// 	let use_custom_font = flags.contains(PrintAttributeFlags::CUSTOM_FONT);
// 	
// 	let font = if use_custom_font { Font::new((&env.memory[0x5600..=0x5dff]).try_into()?) } else { Font::SYSTEM };
// 	
// }
// 
// pub fn print<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (text, x, y, color): (String, Option<i16>, Option<i16>, Option<i16>)) -> Result<(), RuntimeError> {
// 	info!("[print] {}", text);
// 	
// 	let mut env = env.borrow_mut();
// 	
// 	set_cursor_color(&mut env.memory.draw_state(), x, y, color);
// 	
// 	let flags = env.memory.hardware_state().get_print_defaults();
// 	let flags = if flags.contains(PrintAttributeFlags::ENABLE) { flags } else { PrintAttributeFlags::empty() };
// 	
// 	let home_x = env.memory.draw_state().cursor_home_x();
// 	let [cursor_x, cursor_y] = env.memory.draw_state().cursor_position();
// 	
// 	for &letter in text.as_bytes() {
// 		match letter {
// 			..16 => {
// 				// todo
// 			}
// 			16.. => {
// 			
// 			}
// 		}
// 	}
// 	
// 	Ok(())
// }

pub fn cls<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (col): (Option<i16>)) -> Result<(), RuntimeError> {
	let mut env = env.borrow_mut();
	let col = col.unwrap_or(0) as u8 & 0xF;
	let col = (col << 4) | col;
	for byte in env.memory.screen().iter_mut() {
		*byte = col;
	}
	*env.memory.draw_state().cursor_position() = [0, 0];
	*env.memory.draw_state().clip_rect() = [0, 0, 128, 128];
	Ok(())
}

pub fn cursor<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (x, y, col): (Option<i16>, Option<i16>, Option<i16>)) -> Result<(u8, u8, u8), RuntimeError> {
	let mut env = env.borrow_mut();
	let prev_cursor = *env.memory.draw_state().cursor_position();
	let prev_color = *env.memory.draw_state().pen_color();
	
	set_cursor_color(&mut env.memory.draw_state(), x.or(Some(0)), y.or(Some(0)), col);
	
	Ok((prev_cursor[0], prev_cursor[1], prev_color))
}

pub fn color<A: Allocator + Clone + 'static>(env: EnvHandle<A>, val: Option<i16>) -> Result<u8, RuntimeError> {
	let mut env = env.borrow_mut();
	let old = *env.memory.draw_state().pen_color();
	if let Some(val) = val { *env.memory.draw_state().pen_color() = val as u8; }
	Ok(old)
}

pub fn camera<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (x, y): (Option<i16>, Option<i16>)) -> Result<(i16, i16), RuntimeError> {
	let mut env = env.borrow_mut();
	let old = (env.memory.read_u16_le(0x5f28).cast_signed(), env.memory.read_u16_le(0x5f2a).cast_signed());
	env.memory.write_u16_le(0x5f28, x.unwrap_or(0).cast_unsigned());
	env.memory.write_u16_le(0x5f2a, y.unwrap_or(0).cast_unsigned());
	Ok(old)
}

pub fn clip<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (x, y, w, h, clip_previous): (Option<u8>, Option<u8>, Option<u8>, Option<u8>, Option<bool>)) -> Result<(u8, u8, u8, u8), RuntimeError> {
	let mut env = env.borrow_mut();
	let [x_begin_old, y_begin_old, x_end_old, y_end_old] = *env.memory.draw_state().clip_rect();
	
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
		
		*env.memory.draw_state().clip_rect() = [x_begin, y_begin, x_end.min(128), y_end.min(128)];
	}else{
		*env.memory.draw_state().clip_rect() = [0, 0, 128, 128];
	}
	
	Ok((x_begin_old, y_begin_old, x_end_old, y_end_old))
}

pub fn pal<A: Allocator + Clone + 'static>(env: EnvHandle<A>, args: Variadic<Vec<Value>>) -> Result<(), RuntimeError> {
	let argc = args.len();
	assert!(argc >= 1 && argc <= 3, "Invalid number of arguments");
	
	let mut env = env.borrow_mut();
	
	if let Value::Table(new_pal) = args[0] {
		let pal_idx = if argc > 1 && let Some(p) = args[1].to_number() { p.to_integer() as u8 } else { 0 };
		let pal_base = env.memory.base_addr_palette(pal_idx) as usize;
		for (idx, col) in new_pal {
			if let Some(k) = idx.to_number() && let Some(v) = col.to_number() {
				env.memory[pal_base + k.to_integer().rem_euclid(16) as usize] = v.to_integer() as u8;
			}
		}
	} else if let Some(idx) = args[0].to_number() && let Some(col) = args[1].to_number() {
		let pal_idx = if argc > 2 && let Value::Number(p) = args[2] { p.to_integer() as u8 } else { 0 };
		let pal_base = env.memory.base_addr_palette(pal_idx) as usize;
		env.memory[pal_base + (idx.to_integer() % 16) as usize] = col.to_integer() as u8;
	} else {
		panic!("Invalid arguments");
	}
	
	Ok(())
}