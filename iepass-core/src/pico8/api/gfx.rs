use alloc::vec::Vec;
use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, RuntimeError, Value, Variadic};

use crate::pico8::Runtime;

pub fn install_pico8_gfx<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("camera", camera::callback::<A>(ctx));
	ctx.set_global("color", color::callback::<A>(ctx));
	ctx.set_global("clip", clip::callback::<A>(ctx));
	ctx.set_global("pal", pal::callback::<A>(ctx));
}

#[api_callback]
pub fn camera<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<i16>, y: Option<i16>) -> Result<(i16, i16), RuntimeError> {
	let old = (rt.memory.read(0x5f28), rt.memory.read(0x5f2a));
	rt.memory.write(0x5f28, x.unwrap_or(0));
	rt.memory.write(0x5f2a, y.unwrap_or(0));
	Ok(old)
}

#[api_callback]
pub fn color<A: Allocator + 'static>(rt: &mut Runtime<A>, val: Option<u32>) -> Result<u8, RuntimeError> {
	let old = rt.memory[0x5f25];
	if let Some(val) = val { rt.memory[0x5f25] = val as u8; }
	Ok(old)
}

#[api_callback]
pub fn clip<A: Allocator + 'static>(rt: &mut Runtime<A>, x: Option<u8>, y: Option<u8>, w: Option<u8>, h: Option<u8>, clip_previous: Option<bool>) -> Result<(u8, u8, u8, u8), RuntimeError> {
	let [x_begin_old, y_begin_old, x_end_old, y_end_old] = rt.memory[0x5f20..=0x5f23].try_into().unwrap();
	
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
		
		rt.memory[0x5f20] = x_begin;
		rt.memory[0x5f21] = y_begin;
		rt.memory[0x5f22] = x_end.min(128);
		rt.memory[0x5f23] = y_end.min(128);
	} else {
		rt.memory[0x5f20] = 0;
		rt.memory[0x5f21] = 0;
		rt.memory[0x5f22] = 128;
		rt.memory[0x5f23] = 128;
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