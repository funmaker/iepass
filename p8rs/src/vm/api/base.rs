use core::fmt::Write;
use alloc::format;
use bitflags::bitflags;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Error, Execution, Function, RuntimeError, Stack, String, Table, Value};
use p8rs_types::p8num::{P8Num, P8NumStringConversionFlags};
use crate::vm::Runtime;
use crate::vm::traceback::write_traceback_entries;

pub fn load(ctx: Context) {
	ctx.set_global(b"assert", assert::callback(ctx));
	ctx.set_global(b"select", select::callback(ctx));
	ctx.set_global(b"rawget", rawget::callback(ctx));
	ctx.set_global(b"rawlen", rawlen::callback(ctx));
	ctx.set_global(b"rawset", rawset::callback(ctx));
	ctx.set_global(b"rawequal", rawequal::callback(ctx));
	ctx.set_global(b"getmetatable", getmetatable::callback(ctx));
	ctx.set_global(b"setmetatable", setmetatable::callback(ctx));
	ctx.set_global(b"trace", trace::callback(ctx));
	ctx.set_global(b"stat", stat::callback(ctx));
	ctx.set_global(b"type", r#type::callback(ctx));
	ctx.set_global(b"tostring", tostring::callback(ctx));
	ctx.set_global(b"tostr", tostr::callback(ctx));
	ctx.set_global(b"tonum", tonum::callback(ctx));
	ctx.set_global(b"printh", printh::callback(ctx));
	ctx.set_global(b"time", time::callback(ctx));
	ctx.set_global(b"t", time::callback(ctx));
}

#[api_callback]
pub fn assert<'gc>(ctx: Context<'gc>, check: Value<'gc>, error: Option<Value<'gc>>) -> Result<Value<'gc>, Value<'gc>> {
	match check {
		Value::Nil | Value::Boolean(false) => Err(error.unwrap_or(String::from_static(&ctx, b"assertion failed!").into())),
		value => Ok(value)
	}
}

#[api_callback]
pub fn select<'gc>(ctx: Context<'gc>, stack: Stack<'gc, '_>) -> Result<Value<'gc>, Value<'gc>> {
	let len = stack.len() - 1;
	let pos = stack.get(0);
	
	if let Value::String(pos) = pos && pos == "#" {
		Ok((len as i16).into())
	} else if let Some(pos) = pos.to_number() {
		let int = pos.to_integer() as usize;
		if int >= len { Ok(Value::Nil) }
		else { Ok(stack.get(int + 1)) }
	} else {
		Err(String::from_static(&ctx, b"bad argument #0 to 'select'").into())
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
pub fn rawequal<'gc>(lhs: Value<'gc>, rhs: Value<'gc>) -> bool {
	match (lhs, rhs) {
		(Value::Nil, Value::Nil) => true,
		(Value::Boolean(a), Value::Boolean(b)) => a == b,
		(Value::Number(a), Value::Number(b)) => a == b,
		(Value::String(a), Value::String(b)) => a == b,
		(Value::Function(a), Value::Function(b)) => a == b,
		(Value::Thread(a), Value::Thread(b)) => a == b,
		(Value::Table(a), Value::Table(b)) => a == b,
		(Value::UserData(a), Value::UserData(b)) => a == b,
		_ => false
	}
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
		(coroutine, message, skip) = (Value::Nil, coroutine, message);
	}
	
	let skip = if let Value::Number(skip) = skip { skip.to_integer() } else { 1 };
	let mut buf = alloc::string::String::new();
	
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
pub fn stat<'gc>(rt: &mut Runtime, stat_cmd: i16) -> P8Num {
	match stat_cmd {
		7 => P8Num::from(rt.target_fps.cast_signed()),
		11 => P8Num::ONE,
		other => {
			rt.callbacks().printh(format!("[stat] {other} cmd not implemented.").as_bytes(), None, None, None);
			P8Num::ZERO
		}
	}
}

#[api_callback]
pub fn r#type<'gc>(ctx: Context<'gc>, val: Option<Value<'gc>>) -> Option<String<'gc>> {
	val.map(|val| match val {
		Value::Nil => String::from_static(&ctx, b"nil"),
		Value::Boolean(_) => String::from_static(&ctx, b"boolean"),
		Value::Number(_) => String::from_static(&ctx, b"number"),
		Value::String(_) => String::from_static(&ctx, b"string"),
		Value::Table(_) => String::from_static(&ctx, b"table"),
		Value::Function(_) => String::from_static(&ctx, b"function"),
		Value::Thread(_) => String::from_static(&ctx, b"thread"),
		Value::UserData(_) => String::from_static(&ctx, b"userdata"),
	})
}

#[api_callback]
pub fn tostring<'gc>(ctx: Context<'gc>, val: Option<Value<'gc>>) -> Result<String<'gc>, Value<'gc>> {
	let val = val.ok_or_else(|| String::from_static(&ctx, "bad argument #0 to 'tostring' (value expected)"))?;
	
	Ok(match val {
		Value::Nil => String::from_static(&ctx, b"nil"),
		Value::Boolean(true) => String::from_static(&ctx, b"true"),
		Value::Boolean(false) => String::from_static(&ctx, b"false"),
		Value::Number(num) => String::from_slice(&ctx, num.to_str_fmt(P8NumStringConversionFlags::empty()).as_ref().as_bytes()),
		Value::String(str) => str,
		Value::Function(Function::Closure(cls)) => String::from_buffer(&ctx, format!("function: {:p}", cls.into_inner()).into_boxed_str().into_boxed_bytes()),
		Value::Function(Function::Callback(clb)) => String::from_buffer(&ctx, format!("function: {:p}", clb.into_inner()).into_boxed_str().into_boxed_bytes()),
		Value::Thread(thread) => String::from_buffer(&ctx, format!("thread: {:p}", thread.into_inner()).into_boxed_str().into_boxed_bytes()),
		Value::Table(tab) => String::from_buffer(&ctx, format!("table: {:p}", tab.into_inner()).into_boxed_str().into_boxed_bytes()),
		Value::UserData(ud) => String::from_buffer(&ctx, format!("userdata: {:p}", ud.into_inner()).into_boxed_str().into_boxed_bytes()),
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
	let text = match val {
		Value::Boolean(false) => b"0",
		Value::Boolean(true) => b"1",
		Value::String(str) => str.as_bytes(),
		Value::Number(num) => return Some(num),
		_ => return None,
	};
	
	let flags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
	
	if flags.contains(NumberConversionFlags::FORCE_HEX) {
		let mut num = 0_u32;
		
		for char in text {
			num = num.wrapping_shl(4);
			
			match char {
				b'0'..=b'9' => num = num.wrapping_add((char - b'0') as _),
				b'a'..=b'f' => num = num.wrapping_add((char - b'a' + 10) as _),
				b'A'..=b'F' => num = num.wrapping_add((char - b'A' + 10) as _),
				_ => continue,
			}
		}
		
		if !flags.contains(NumberConversionFlags::SHIFT_16) {
			num = num.wrapping_shl(16);
		}
		
		Some(P8Num::from_raw(num.cast_signed()))
	} else if flags.contains(NumberConversionFlags::SHIFT_16) {
		let mut num = 0_u32;
		
		let (text, negative) = match text {
			[b'-', rest @ ..] => (rest, true),
			[b'+', rest @ ..] => (rest, false),
			[rest @ ..] => (rest, false),
		};
		
		for char in text {
			match char {
				b'0'..=b'9' => {
					num = num.wrapping_mul(10);
					num = num.wrapping_add((char - b'0') as _);
				}, 
				_ => break,
			}
		}
		
		let num = P8Num::from_raw(num.cast_signed());
		
		Some(if negative { -num } else { num })
	} else {
		let (text, negative) = match text {
			[b'-', rest @ ..] => (rest, true),
			[b'+', rest @ ..] => (rest, false),
			[rest @ ..] => (rest, false),
		};
		
		let res = match text {
			[b'0', b'x' | b'X', text @ ..] => P8Num::from_ascii_radix(text, 16),
			[b'0', b'b' | b'B', text @ ..] => P8Num::from_ascii_radix(text, 2),
			text => P8Num::from_ascii_radix(text, 10),
		};
		
		if res.is_err() && flags.contains(NumberConversionFlags::ZERO_ON_FAIL) {
			Some(P8Num::ZERO)
		} else {
			res.ok()
			   .map(|num| if negative { -num } else { num })
		}
	}
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

bitflags! {
    pub struct NumberConversionFlags: u8 {
		/// Read using hexadecimal notation, without requiring the "0x" prefix.
		/// Note: Non-hexadecimal characters, including '.' and '-', are taken to be '0'.
        const FORCE_HEX    = 1 << 0;
		
		/// Shift the value right 16 bits to create a 16.16 fixed-point number.
		/// This works with all formats, even booleans: true becomes 0x.0001.
        const SHIFT_16     = 1 << 1;
		
		/// When value cannot be converted to a number, return 0 instead of nothing.
        const ZERO_ON_FAIL = 1 << 2;
    }
}

