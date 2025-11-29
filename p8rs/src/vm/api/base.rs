use core::fmt::Write;

use crate::vm::numeric::{number_from_ascii, NumberConversionFlags};
use crate::vm::Runtime;
use anyhow::anyhow;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Execution, IntoValue, RuntimeError, String, Value};
use p8rs_types::p8num::{P8Num, P8NumStringConversionFlags};
use crate::vm::traceback::write_traceback_entries;

pub fn install_pico8_base(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	// TODO: Remove
	p8rs_piccolo::stdlib::load_base(ctx);
	p8rs_piccolo::stdlib::load_table(ctx);
	
	ctx.set_global("tostr", tostr::callback(ctx));
	ctx.set_global("tonum", tonum::callback(ctx));
	ctx.set_global("printh", printh::callback(ctx));
	ctx.set_global("type", get_type::callback(ctx));
	ctx.set_global("btn", btn::callback(ctx));
	ctx.set_global("stat", stat::callback(ctx));
	ctx.set_global("trace", trace::callback(ctx));
}

#[api_callback]
pub fn trace<'gc>(ex: Execution<'gc, '_>, ctx: Context<'gc>, mut coroutine: Value<'gc>, mut message: Value<'gc>, mut skip: Value<'gc>) -> Result<String<'gc>, RuntimeError<'gc>> {
	if skip.is_nil() { // if up to 2 args passed, skip coroutine arg
		skip = message;
		message = coroutine;
		coroutine = Value::Nil;
	}
	
	let mut buf: alloc::string::String = alloc::string::String::new();
	
	if let Value::String(first_line) = message {
		write!(&mut buf, "{}\n", first_line)?;
	}
	
	write!(&mut buf, "stack traceback:\n")?;
	
	if let Value::Thread(coroutine) = coroutine && coroutine != ex.current_thread().thread {
		write!(&mut buf, "\t**getting trace of different threads is not currently supported**\n")?;
	} else {
		let trace = ex.traceback(&ctx);
		let entries = trace.entries();
		
		// always skip the last entry (p8_prelog.lua)
		let entries = &entries[0..entries.len().saturating_sub(1)];
		
		let skip = if let Value::Number(skip) = skip { skip.to_integer() } else { 1 };
		
		if skip == 0 {
			write!(&mut buf, "\t[C]: in function 'trace'\n")?;
		}
		
		if skip >= 0 {
			write_traceback_entries(&mut buf, entries.iter().skip((skip as usize).saturating_sub(1)))?;
		}
		
	}
	
	buf.pop(); // remove last newline
	
	Ok(String::from_buffer(ctx.mutation(), buf.into_bytes().into_boxed_slice()))
}


#[api_callback]
pub fn stat<'gc>(rt: &mut Runtime, stat_cmd: i16) -> Result<Option<Value<'gc>>, RuntimeError<'gc>> {
	Ok(match stat_cmd {
		7 => { Some(Value::Number(P8Num::from(rt.target_fps.cast_signed()))) }
		other => { return Err(anyhow!("stat({}) not implemented!", other).into())}
	})
}

#[api_callback]
pub fn btn<'gc>(rt: &mut Runtime, btn_idx: Option<i16>, player_idx: Option<i16>) -> Result<Option<Value<'gc>>, RuntimeError<'gc>> {
	match btn_idx {
		None => Ok(Some(Value::Number(P8Num::from((rt.buttons.get_bits_for_player(0) as u16 | (rt.buttons.get_bits_for_player(1) as u16) << 8).cast_signed())))),
		Some(button_idx) => {
			let player_idx = player_idx.unwrap_or(0);
			if player_idx < 0 || player_idx > 7 { return Ok(Some(Value::Boolean(false))) }
			
			Ok(Some(Value::Boolean(rt.buttons.is_down(player_idx as usize, button_idx as usize))))
		}
	}
}

// TODO: btnp - observe 0x5f5c and 0x5f5d

#[api_callback]
pub fn get_type<'gc>(ctx: Context<'gc>, val: Value<'gc>) -> Result<Option<Value<'gc>>, RuntimeError<'gc>> {
	Ok(Some(match val {
		Value::Nil => "nil".into_value(ctx),
		Value::Boolean(_) => "boolean".into_value(ctx),
		Value::Number(_) => "number".into_value(ctx),
		Value::String(_) => "string".into_value(ctx),
		Value::Table(_) => "table".into_value(ctx),
		Value::Function(_) => "function".into_value(ctx),
		Value::Thread(_) => "thread".into_value(ctx),
		Value::UserData(_) => "userdata".into_value(ctx),
	}))
}

#[api_callback]
pub fn tostr<'gc>(ctx: Context<'gc>, val: Value<'gc>, opts: Option<Value<'gc>>) -> Result<Option<Value<'gc>>, RuntimeError<'gc>> {
	let flags = P8NumStringConversionFlags::from_bits_truncate(match opts {
		Some(Value::Boolean(true)) => 1,
		Some(Value::Number(num)) => num.to_integer() as u8,
		_ => 0,
	});
	
	let result = match val {
		Value::Nil => "[nil]".into_value(ctx),
		Value::Boolean(x) => if x { "true" } else { "false" }.into_value(ctx),
		Value::Number(num) => String::from_slice(ctx.mutation(), num.to_str_fmt(flags).as_ref().as_bytes()).into_value(ctx),
		Value::String(s) => s.into_value(ctx),
		Value::Table(_) => "[table]".into_value(ctx),
		Value::Function(_) => "[function]".into_value(ctx),
		Value::Thread(_) => "[thread]".into_value(ctx),
		Value::UserData(_) => "[userdata]".into_value(ctx),
	};
	
	Ok(Some(result))
}

#[api_callback]
pub fn tonum<'gc>(val: String, opts: Option<u8>) -> Option<Value<'gc>> {
	let flags: NumberConversionFlags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
	number_from_ascii(&val, flags).map(|x| Value::Number(x)).ok()
}

#[api_callback]
pub fn printh(rt: &mut Runtime, text: String, filename: Option<String>, overwrite: Option<bool>, save_to_desktop: Option<bool>) {
	rt.callbacks.printh(&text, filename.as_deref(), overwrite, save_to_desktop);
}




