use super::{set_global_callback_ctx, set_global_callback_simple};
use crate::pico8::numeric::{number_from_string, NumberConversionFlags};
use alloc::string::String;
use p8rs_piccolo::{Context, IntoValue, RuntimeError, Value};
use p8rs_types::p8num::P8NumStringConversionFlags;

pub fn install_pico8_base(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	p8rs_piccolo::stdlib::load_base(ctx);
	
	set_global_callback_ctx("tostr", ctx, tostr);
	set_global_callback_simple("tonum", ctx, tonum);
	set_global_callback_simple("printh", ctx, printh);
	set_global_callback_simple("print", ctx, print);
}

pub fn tostr<'gc>(ctx: Context<'gc>, (val, opts): (Value<'gc>, Option<Value<'gc>>)) -> Result<Option<Value<'gc>>, RuntimeError> {
	let flags = P8NumStringConversionFlags::from_bits_truncate(match opts {
		Some(Value::Boolean(true)) => 1,
		Some(Value::Number(num)) => num.to_integer() as u8,
		_ => 0,
	});
	
	let result = match val {
		Value::Nil => "[nil]".into_value(ctx),
		Value::Boolean(x) => if x { "true" } else { "false" }.into_value(ctx),
		Value::Number(num) => p8rs_piccolo::string::String::from_slice(ctx.mutation(), num.to_str_fmt(flags).as_ref().as_bytes()).into_value(ctx),
		Value::String(s) => s.into_value(ctx),
		Value::Table(_) => "[table]".into_value(ctx),
		Value::Function(_) => "[function]".into_value(ctx),
		Value::Thread(_) => "[thread]".into_value(ctx),
		Value::UserData(_) => "[userdata]".into_value(ctx),
	};
	
	Ok(Some(result))
}


pub fn tonum<'gc>((val, opts): (String, Option<u8>)) -> Result<Option<Value<'gc>>, RuntimeError> {
	let flags: NumberConversionFlags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
	let conversion = number_from_string(val.as_str(), flags);
	if conversion.is_ok() {
		Ok(Some(conversion.unwrap()))
	}else{
		Ok(None)
	}
}

pub fn printh((text, filename, _overwrite, _save_to_desktop): (String, Option<String>, Option<bool>, Option<bool>)) -> Result<(), RuntimeError> {
	if let Some(filename_str) = filename {
		info!("[printh/{}] {}", filename_str, text);
	} else {
		info!("[printh] {}", text);
	}
	Ok(())
}

pub fn print((text, _x, _y, _color): (String, Option<i16>, Option<i16>, Option<u8>)) -> Result<(), RuntimeError> {
	info!("[print] {}", text);
	// todo: implement on-screen printing
	Ok(())
}




