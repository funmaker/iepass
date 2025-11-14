use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, RuntimeError, Variadic};

use crate::vm::Runtime;

pub fn install_pico8_memory<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("peek", peek::callback::<A>(ctx));
	ctx.set_global("poke", poke::callback::<A>(ctx));
}

#[api_callback]
pub fn poke<A: Allocator + 'static>(rt: &mut Runtime<A>, addr: i16, bytes: Variadic<alloc::vec::Vec<u8>>) -> Result<(), RuntimeError> {
	let addr = addr.cast_unsigned() as usize;
	
	if bytes.is_empty() {
		rt.memory[addr] = 0;
	} else {
		rt.memory[addr..addr+bytes.len()].copy_from_slice(bytes.as_slice());
	}
	
	Ok(())
}

#[api_callback]
pub fn peek<A: Allocator + 'static>(rt: &mut Runtime<A>, addr: i16, n: Option<i16>) -> Result<Variadic<alloc::vec::Vec<u8>>, RuntimeError> {
	let addr = addr.cast_unsigned() as usize;
	let n = n.map(|v| if v.is_negative() { 0 } else { v as usize }).unwrap_or(1);
	
	let bytes = rt.memory.iter().skip(addr).take(n).copied().collect();
	
	Ok(Variadic(bytes))
}