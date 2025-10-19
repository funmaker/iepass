use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::RefCell;
use p8rs_piccolo::{Context, RuntimeError, Variadic};

use super::{set_global_callback_env, EnvHandle};
use crate::pico8::env::Env;

pub fn install_pico8_memory<A: Allocator + Clone + 'static>(env: Rc<RefCell<Env<A>>>, ctx: Context) {
	set_global_callback_env("peek", ctx, env.clone(), peek);
	set_global_callback_env("poke", ctx, env.clone(), poke);
}

pub fn poke<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (addr, bytes): (i16, Variadic<Vec<u8>>)) -> Result<(), RuntimeError> {
	let mut env = env.borrow_mut();
	let addr = addr.cast_unsigned() as usize;
	
	if bytes.is_empty() {
		env.memory[addr] = 0;
	} else {
		env.memory[addr..addr+bytes.len()].copy_from_slice(bytes.as_slice());
	}
	
	Ok(())
}

pub fn peek<A: Allocator + Clone + 'static>(env: EnvHandle<A>, (addr, n): (i16, Option<i16>)) -> Result<Variadic<Vec<u8>>, RuntimeError> {
	let env = env.borrow();
	let addr = addr.cast_unsigned() as usize;
	let n = n.map(|v| if v.is_negative() { 0 } else { v as usize }).unwrap_or(1);
	
	let bytes = env.memory.iter().skip(addr).take(n).copied().collect();
	
	Ok(Variadic(bytes))
}