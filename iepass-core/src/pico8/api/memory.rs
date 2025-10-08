use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::RefCell;
use piccolo::{Context, RuntimeError, Table, Value, Variadic};
use crate::pico8::env::Env;
use super::{set_global_callback_ctx_env, set_global_callback_env, EnvHandle};

pub fn install_pico8_memory<A: Allocator + Clone + 'static>(env: Rc<RefCell<Env<A>>>, ctx: Context) {
	set_global_callback_ctx_env("peek", ctx, env.clone(), peek);
	set_global_callback_env("poke", ctx, env.clone(), poke);
}

pub fn poke<A: Allocator + Clone + 'static>(env: EnvHandle<A>,  (addr, mut bytes): (u32, Variadic<alloc::vec::Vec<u8>>)) -> Result<(), RuntimeError> {
	let mut env = env.borrow_mut();
	if bytes.is_empty() { bytes.push(0) }
	for (pos, byte) in bytes.into_iter().enumerate() {
		env.memory[addr as usize + pos] = byte;
	}
	Ok(())
}

pub fn peek<A: Allocator + Clone + 'static>(ctx: Context, env: EnvHandle<A>,  (addr, n): (u32, Option<u32>)) -> Result<Value, RuntimeError> {
	let env = env.borrow();
	let n = n.unwrap_or(1);
	if n == 1 { return Ok(Value::Integer(env.memory[addr as usize] as i64)); }
	
	let table = Table::new(&ctx);
	for (pos, byte) in env.memory[addr as usize .. (addr + n) as usize].iter().enumerate() {
		table.set(ctx, pos as u32 + 1, byte)?;
	}
	Ok(Value::Table(table))
}