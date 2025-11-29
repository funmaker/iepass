// Based on https://www.lexaloffle.com/bbs/?pid=153638#p

use p8rs_macros::{api_callback, p8};
use p8rs_piccolo::{Context, Value};
use p8rs_types::p8num::P8Num;
use crate::vm::Runtime;

pub fn install_pico8_rnd(ctx: Context) {
	ctx.set_global("rnd", rnd::callback(ctx));
	ctx.set_global("srand", srand::callback(ctx));
}

#[api_callback]
pub fn rnd<'gc>(ctx: Context<'gc>, rt: &mut Runtime, limit: Option<Value<'gc>>) -> Value<'gc> {
	let limit = match limit {
		Some(Value::Number(limit)) if limit != p8!(0) => limit.to_raw().cast_unsigned(),
		None => p8!(1).to_raw().cast_unsigned(),
		Some(Value::Table(table)) => {
			let len = table.length();
			if len == 0 {
				return Value::Nil
			} else {
				let value = (rnd_impl(rt) >> 8) % len as u32;
				return table.get_value(ctx, value as i16 + 1)
			}
		},
		_ => return p8!(0).into(),
	};
	
	let value = rnd_impl(rt);
	
	P8Num::from_raw((value % limit).cast_signed()).into()
}

fn rnd_impl(rt: &mut Runtime) -> u32 {
	let mut ms = rt.memory.machine_state();
	let rng_state = ms.rng_state();
	let mut hi = u32::from_le_bytes(rng_state[0..4].try_into().unwrap());
	let mut lo = u32::from_le_bytes(rng_state[4..8].try_into().unwrap());
	
	hi = hi.rotate_left(16);
	hi = hi.wrapping_add(lo);
	lo = lo.wrapping_add(hi);
	
	rng_state.copy_from_slice([hi.to_le_bytes(), lo.to_le_bytes()].as_flattened());
	
	hi
}

#[api_callback]
pub fn srand<'gc>(ctx: Context<'gc>, rt: &mut Runtime, seed: Option<P8Num>) {
	let seed = seed.map(P8Num::to_raw).unwrap_or(0).cast_unsigned() & 0x7fff_ffff;
	let (hi, lo) = match seed {
		0 => (0x6000_9755, 0xdead_beef),
		seed => (seed ^ 0xbead_29ba, seed),
	};
	
	rt.memory
	  .machine_state()
	  .rng_state()
	  .copy_from_slice([hi.to_le_bytes(), lo.to_le_bytes()].as_flattened());
	
	for _ in 0..32 {
		rnd(ctx, rt, None);
	}
}
