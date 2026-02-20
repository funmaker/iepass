use p8rs_piccolo::{Error, Value};
use crate::piccolo::Stack;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, IntoValue};
use p8rs_types::p8num::P8Num;
use crate::vm::memory::MemoryAccess;
use crate::vm::numeric::{number_from_ascii, NumberConversionFlags};
use crate::vm::Runtime;

pub fn load(ctx: Context) {
	ctx.set_global("peek", peek::callback(ctx));
	ctx.set_global("peek2", peek2::callback(ctx));
	ctx.set_global("peek4", peek4::callback(ctx));
	ctx.set_global("poke", poke::callback(ctx));
	ctx.set_global("poke2", poke2::callback(ctx));
	ctx.set_global("poke4", poke4::callback(ctx));
}

/// Inclusive range, end >= start.
fn poke_mmio_range(rt: &mut Runtime, start: u16, end: u16) {
	let handlers: [(u16, fn(&mut Runtime, u8)); _] = [
		(0x5f24, |rt, val| rt.set_cursor_home(val as i16)),
		(0x5f26, |rt, val| rt.set_cursor_x(val as i16)),
		(0x5f27, |rt, val| rt.set_cursor_y(val as i16)),
	];
	
	if end >= 0x5f24 && start <= 0x5f27 {
		for (addr, handler) in handlers {
			if addr >= start && addr <= end {
				handler(rt, rt.memory[addr as usize]);
			}
		}
	}
}


/// Inclusive cyclic range. If `end == start`, 1 byte is triggered. If `end == start - 1`, all memory is triggered.
fn poke_mmio_cyclic(rt: &mut Runtime, start: u16, end: u16) {
	if end > start {
		poke_mmio_range(rt, start, end);
	} else {
		poke_mmio_range(rt, start, 0xffff);
		poke_mmio_range(rt, 0, end);
	}
}

fn parse_poke_args<'gc, 'a>(ctx: Context<'gc>, stack: &mut Stack<'gc, 'a>) -> Result<(u16, usize, bool), Error<'gc>> {
	let base = stack.pop_front().ok_or_else(|| "[poke]: Addr argument required".into_value(ctx))?;
	let base = match base {
		Value::Number(num) => num.to_integer().cast_unsigned(),
		_ => return Err("[poke]: Addr argument must be a number".into_value(ctx).into()),
	};
	
	let count = stack.len().max(1);
	
	Ok((base, count, stack.is_empty()))
}

#[api_callback]
pub fn poke<'gc, 'a>(rt: &mut Runtime, ctx: Context<'gc>, mut stack: Stack<'gc, 'a>) -> Result<(), Error<'gc>> {
	let (base, count, no_data) = parse_poke_args(ctx, &mut stack)?;
	
	if no_data {
		rt.memory.write(base, 0u8);
	} else {
		for i in 0..count {
			let addr = base.wrapping_add(i as u16);
			let val = match stack.get(i) {
				Value::Number(v) => v.to_integer() as u8,
				Value::String(v) => number_from_ascii(v.as_bytes(), NumberConversionFlags::ZERO_ON_FAIL).unwrap().to_integer() as u8,
				_ => 0,
			};
			rt.memory.write(addr, val);
		}
	}
	
	poke_mmio_cyclic(rt, base, base.wrapping_add((count - 1).min(0xffff) as u16));
	Ok(())
}

#[api_callback]
pub fn poke2<'gc, 'a>(rt: &mut Runtime, ctx: Context<'gc>, mut stack: Stack<'gc, 'a>) -> Result<(), Error<'gc>> {
	let (base, count, no_data) = parse_poke_args(ctx, &mut stack)?;
	
	if no_data {
		rt.memory.write(base, 0u16);
	} else {
		for i in 0..count {
			let addr = base.wrapping_add((i*2) as u16);
			let val = match stack.get(i) {
				Value::Number(v) => v.to_integer(),
				Value::String(v) => number_from_ascii(v.as_bytes(), NumberConversionFlags::ZERO_ON_FAIL).unwrap().to_integer(),
				_ => 0,
			};
			rt.memory.write(addr, val);
		}
	}
	
	poke_mmio_cyclic(rt, base, base.wrapping_add((count*2 - 1).min(0xffff) as u16));
	Ok(())
}

#[api_callback]
pub fn poke4<'gc, 'a>(rt: &mut Runtime, ctx: Context<'gc>, mut stack: Stack<'gc, 'a>) -> Result<(), Error<'gc>> {
	let (base, count, no_data) = parse_poke_args(ctx, &mut stack)?;
	
	if no_data {
		rt.memory.write(base, 0u32);
	} else {
		for i in 0..count {
			let addr = base.wrapping_add((i*4) as u16);
			let val = match stack.get(i) {
				Value::Number(v) => v.to_raw(),
				Value::String(v) => number_from_ascii(v.as_bytes(), NumberConversionFlags::ZERO_ON_FAIL).unwrap().to_raw(),
				_ => 0,
			};
			rt.memory.write(addr, val);
		}
	}
	
	poke_mmio_cyclic(rt, base, base.wrapping_add((count*4 - 1).min(0xffff) as u16));
	Ok(())
}

#[api_callback]
pub fn peek(rt: &mut Runtime, mut stack: Stack, addr: u16, n: Option<u16>) {
	let n = match n {
		None => 1,
		Some(n) if n > 0 => n,
		_ => return,
	};
	stack.clear();
	stack.extend(
		rt.memory
		  .iter()
		  .copied()
		  .cycle()
		  .skip(addr as usize)
		  .take(n as usize)
		  .map(Value::from)
	);
}

#[api_callback]
pub fn peek2(rt: &mut Runtime, mut stack: Stack, addr: u16, n: Option<u16>) {
	let n = match n {
		None => 1,
		Some(n) if n > 0 => n,
		_ => return,
	};
	
	stack.clear();
	stack.extend(
		rt.memory
		  .iter()
		  .copied()
		  .cycle()
		  .skip(addr as usize)
		  .array_chunks()
		  .take(n as usize)
		  .map(i16::from_le_bytes)
		  .map(Value::from)
	);
}

#[api_callback]
pub fn peek4(rt: &mut Runtime, mut stack: Stack, addr: u16, n: Option<u16>) {
	let n = match n {
		None => 1,
		Some(n) if n > 0 => n,
		_ => return,
	};
	
	stack.clear();
	stack.extend(
		rt.memory
		  .iter()
		  .copied()
		  .cycle()
		  .skip(addr as usize)
		  .array_chunks()
		  .take(n as usize)
		  .map(i32::from_le_bytes)
		  .map(P8Num::from_raw)
		  .map(Value::from)
	);
}