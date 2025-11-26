use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Variadic};

use crate::vm::Runtime;

pub fn install_pico8_memory<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("peek", peek::callback::<A>(ctx));
	ctx.set_global("poke", poke::callback::<A>(ctx));
}

#[api_callback]
pub fn poke<A: Allocator + 'static>(rt: &mut Runtime<A>, addr: i16, bytes: Variadic<alloc::vec::Vec<u8>>) {
	let addr = addr.cast_unsigned() as usize;
	let range = addr..(addr + bytes.len().max(1));
	
	if range.end > 0x5f24 && range.start <= 0x5f27 {
		let handlers: &[(usize, fn(&mut Runtime<A>, u8))] = &[
			(0x5f24, |rt, val| rt.set_cursor_home(val as i16)),
			(0x5f26, |rt, val| rt.set_cursor_x(val as i16)),
			(0x5f27, |rt, val| rt.set_cursor_y(val as i16)),
		];
		
		for &(addr, handler) in handlers {
			if range.contains(&addr) {
				handler(rt, if bytes.is_empty() { 0 } else { bytes[addr - range.start] });
			}
		}
	}
	
	if bytes.is_empty() {
		rt.memory[addr] = 0;
	} else {
		rt.memory[range].copy_from_slice(bytes.as_slice());
	}
}

#[api_callback]
pub fn peek<A: Allocator + 'static>(rt: &mut Runtime<A>, addr: i16, n: Option<i16>) -> Variadic<alloc::vec::Vec<u8>> {
	let addr = addr.cast_unsigned() as usize;
	let n = n.map(|v| if v.is_negative() { 0 } else { v as usize }).unwrap_or(1);
	
	let bytes = rt.memory.iter().skip(addr).take(n).copied().collect();
	
	Variadic(bytes)
}