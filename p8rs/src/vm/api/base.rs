use core::fmt::Write;
use anyhow::anyhow;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Error, Execution, Function, IntoValue, RuntimeError, Stack, String, Table, Value};
use p8rs_types::p8num::{P8Num, P8NumStringConversionFlags};
use crate::vm::numeric::{number_from_ascii, NumberConversionFlags};
use crate::vm::Runtime;
use crate::vm::traceback::write_traceback_entries;

pub fn load(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	// TODO: Remove
	p8rs_piccolo::stdlib::load_base(ctx);
	p8rs_piccolo::stdlib::load_table(ctx);
	
	ctx.set_global("assert", assert::callback(ctx));
	ctx.set_global("select", select::callback(ctx));
	ctx.set_global("rawget", rawget::callback(ctx));
	ctx.set_global("rawlen", rawlen::callback(ctx));
	ctx.set_global("rawset", rawset::callback(ctx));
	ctx.set_global("getmetatable", getmetatable::callback(ctx));
	ctx.set_global("setmetatable", setmetatable::callback(ctx));
	ctx.set_global("trace", trace::callback(ctx));
	ctx.set_global("stat", stat::callback(ctx));
	ctx.set_global("type", r#type::callback(ctx));
	ctx.set_global("tostr", tostr::callback(ctx));
	ctx.set_global("tonum", tonum::callback(ctx));
	ctx.set_global("printh", printh::callback(ctx));
	ctx.set_global("time", time::callback(ctx));
	ctx.set_global("t", time::callback(ctx));
}

#[api_callback]
pub fn assert<'gc>(ctx: Context<'gc>, check: Value<'gc>, error: Option<Value<'gc>>) -> Result<Value<'gc>, Value<'gc>> {
	match check {
		Value::Nil | Value::Boolean(false) => Err(error.unwrap_or(ctx.intern_static(b"assertion failed!").into())),
		value => Ok(value)
	}
}

#[api_callback]
pub fn select<'gc>(ctx: Context<'gc>, stack: Stack<'gc, '_>) -> Result<Value<'gc>, Value<'gc>> {
	let len = stack.len() - 1;
	let count = stack.get(0);
	
	if let Value::String(count) = count && count == "#" {
		Ok((len as i16).into())
	} else if let Some(count) = count.to_number() {
		let int = count.to_integer() as usize;
		if int >= len { Ok(Value::Nil) }
		else { Ok(stack.get(int + 1)) }
	} else {
		Err(ctx.intern_static(b"bad argument #0 to 'select'").into())
	}
}

#[api_callback]
pub fn rawget<'gc>(ctx: Context<'gc>, table: Table<'gc>, key: Value<'gc>) -> Value<'gc> {
	table.get_value(ctx, key)
}

#[api_callback]
pub fn rawlen<'gc>(ctx: Context<'gc>, value: Value<'gc>) -> Result<i16, Value<'gc>> {
	match value {
		Value::Table(value) => Ok(value.length().cast_signed()),
		Value::String(value) => Ok(value.len() as i16),
		_ => Err(ctx.intern_static(b"bad argument #0 to 'rawlen'").into()),
	}
}

#[api_callback]
pub fn rawset<'gc>(ctx: Context<'gc>, table: Table<'gc>, key: Value<'gc>, value: Value<'gc>) -> Result<Value<'gc>, Error<'gc>> {
	table.set(ctx, key, value)?;
	Ok(table.into())
}

#[api_callback]
pub fn getmetatable<'gc>(table: Table<'gc>) -> Option<Table<'gc>> {
	table.metatable()
}

#[api_callback]
pub fn setmetatable<'gc>(ctx: Context<'gc>, table: Table<'gc>, metatable: Option<Table<'gc>>) -> Table<'gc> {
	table.set_metatable(&ctx, metatable);
	table
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
pub fn r#type<'gc>(ctx: Context<'gc>, val: Option<Value<'gc>>) -> Option<Value<'gc>> {
	val.map(|val| match val {
		Value::Nil => "nil".into_value(ctx),
		Value::Boolean(_) => "boolean".into_value(ctx),
		Value::Number(_) => "number".into_value(ctx),
		Value::String(_) => "string".into_value(ctx),
		Value::Table(_) => "table".into_value(ctx),
		Value::Function(_) => "function".into_value(ctx),
		Value::Thread(_) => "thread".into_value(ctx),
		Value::UserData(_) => "userdata".into_value(ctx),
	})
}

#[api_callback]
pub fn tostr<'gc>(ctx: Context<'gc>, val: Option<Value<'gc>>, opts: Option<Value<'gc>>) -> String<'gc> {
	let pointers = !matches!(opts, None | Some(Value::Nil) | Some(Value::Boolean(false)));
	let flags = match opts {
		Some(Value::Boolean(true)) => P8NumStringConversionFlags::HEX,
		Some(Value::Number(num)) => P8NumStringConversionFlags::from_bits_truncate(num.to_integer() as u8),
		_ => P8NumStringConversionFlags::empty(),
	};
	
	let Some(val) = val else {
		return String::from_static(&ctx, b"")
	};
	
	if pointers {
		let with_pointer = match val {
			Value::Function(Function::Closure(cls)) => Some(format!("function: {:p}", cls.into_inner())),
			Value::Function(Function::Callback(clb)) => Some(format!("function: {:p}", clb.into_inner())),
			Value::Table(tab) => Some(format!("table: {:p}", tab.into_inner())),
			Value::UserData(ud) => Some(format!("userdata: {:p}", ud.into_inner())),
			_ => None,
		};
		
		if let Some(val) = with_pointer {
			return String::from_buffer(&ctx, val.into_boxed_str().into_boxed_bytes());
		}
	}
	
	match val {
		Value::Nil => String::from_static(&ctx, b"[nil]"),
		Value::Boolean(true) => String::from_static(&ctx, b"true"),
		Value::Boolean(false) => String::from_static(&ctx, b"false"),
		Value::Number(num) => String::from_slice(&ctx, num.to_str_fmt(flags).as_ref().as_bytes()),
		Value::String(str) => str,
		Value::Table(_) => String::from_static(&ctx, b"[table]"),
		Value::Function(_) => String::from_static(&ctx, b"[function]"),
		Value::Thread(_) => String::from_static(&ctx, b"[thread]"),
		Value::UserData(_) => String::from_static(&ctx, b"[userdata]"),
	}
}

#[api_callback]
pub fn tonum<'gc>(val: Value<'gc>, opts: Option<u8>) -> Option<P8Num> {
	let Value::String(text) = val else { return None };
	let flags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
	
	number_from_ascii(&text, flags).ok()
}

#[api_callback]
pub fn printh<'gc>(ctx: Context<'gc>, rt: &mut Runtime, value: Option<Value<'gc>>, filename: Option<String<'gc>>, overwrite: Option<bool>, save_to_desktop: Option<bool>) {
	if value.is_none() { return; }
	
	let text = tostr(ctx, value, None);
	rt.callbacks().printh(&text, filename.as_deref(), overwrite, save_to_desktop);
}

#[api_callback]
pub fn time(rt: &mut Runtime) -> P8Num {
	P8Num::from_raw(((rt.frame_no as i32 / 60) << 16) | ((rt.frame_no as i32 % 60 << 16) / 60))
}
