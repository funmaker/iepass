use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::RefCell;
use p8rs_piccolo::{Context, Table, Value, Variadic};
use crate::pico8::env::Env;
use super::callback;

pub fn install_pico8_memory<A: Allocator + 'static>(env_orig: Rc<RefCell<Env<A>>>, ctx: Context) {
	
	let env = env_orig.clone();
	ctx.set_global("peek", callback("peek", ctx, move |ctx, (addr, n): (u32, Option<u32>)| {
		let env = env.borrow();
		let n = n.unwrap_or(1);
		if n == 1 { return Value::Integer(env.memory[addr as usize] as i64); }
		
		let table = Table::new(&ctx);
		for (pos, byte) in env.memory[addr as usize .. (addr + n) as usize].iter().enumerate() {
			table.set(ctx, pos as u32 + 1, byte).unwrap();
		}
		Value::Table(table)
	}));
	
	let env = env_orig.clone();
	ctx.set_global("poke", callback("poke", ctx, move |_, (addr, mut bytes): (u32, Variadic<alloc::vec::Vec<u8>>)| {
		let mut env = env.borrow_mut();
		if bytes.is_empty() { bytes.push(0) }
		for (pos, byte) in bytes.into_iter().enumerate() {
			env.memory[addr as usize + pos] = byte;
		}
	}));
	
}