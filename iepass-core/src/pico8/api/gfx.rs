use alloc::rc::Rc;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::cell::RefCell;
use piccolo::{Context, Value, Variadic};
use crate::pico8::env::Env;
use super::callback;

pub fn install_pico8_gfx<A: Allocator + Clone + 'static>(env_orig: Rc<RefCell<Env<A>>>, ctx: Context) {
	
	let env = env_orig.clone();
	ctx.set_global("camera", callback("camera", ctx, move |_, (x, y): (Option<u32>, Option<u32>)| {
		let mut env = env.borrow_mut();
		let old = (env.memory.read_u16_le(0x5f28), env.memory.read_u16_le(0x5f2a));
		env.memory.write_u16_le(0x5f28, x.unwrap_or(0) as u16);
		env.memory.write_u16_le(0x5f2a, y.unwrap_or(0) as u16);
		old
	}));
	
	let env = env_orig.clone();
	ctx.set_global("color", callback("color", ctx, move |_, val: Option<u32>| {
		let mut env = env.borrow_mut();
		let old = env.memory[0x5f25];
		if let Some(val) = val { env.memory[0x5f25] = val as u8; }
		old
	}));
	
	let env = env_orig.clone();
	ctx.set_global("clip", callback("clip", ctx, move |_, (x, y, w, h, clip_previous): (Option<u8>, Option<u8>, Option<u8>, Option<u8>, Option<bool>)| {
		let mut env = env.borrow_mut();
		let [x_begin_old, y_begin_old, x_end_old, y_end_old] = env.memory[0x5f20..=0x5f23].try_into().unwrap();
		
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
			
			env.memory[0x5f20] = x_begin;
			env.memory[0x5f21] = y_begin;
			env.memory[0x5f22] = x_end.min(128);
			env.memory[0x5f23] = y_end.min(128);
		}else{
			env.memory[0x5f20] = 0;
			env.memory[0x5f21] = 0;
			env.memory[0x5f22] = 128;
			env.memory[0x5f23] = 128;
		}
		
		(x_begin_old, y_begin_old, x_end_old, y_end_old)
	}));
	
	
	let env = env_orig.clone();
	ctx.set_global("pal", callback("pal", ctx, move |_, args: Variadic<Vec<Value>>| {
		let argc = args.len();
		assert!(argc >= 1 && argc <= 3, "Invalid number of arguments");
		
		let mut env = env.borrow_mut();
		
		if let Value::Table(t) = args[0] {
			let base = env.memory.base_addr_palette(if argc > 1 && let Value::Integer(p) = args[1] { p as u8 } else { 0 }) as usize;
			for (k, v) in t {
				if let Value::Integer(k) = k && let Value::Integer(v) = v {
					env.memory[base + (k % 16) as usize] = v as u8;
				}
			}
		}else if let Value::Integer(c0) = args[0] && let Value::Integer(c1) = args[1] {
			let base = env.memory.base_addr_palette(if argc > 2 && let Value::Integer(p) = args[2] { p as u8 } else { 0 }) as usize;
			env.memory[base + (c0 % 16) as usize] = c1 as u8;
		}else{
			panic!("Invalid arguments");
		}
	}));
	
}