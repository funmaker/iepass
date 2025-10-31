use crate::pico8::numeric::{number_from_ascii, NumberConversionFlags};
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, IntoValue, RuntimeError, Value, String};
use p8rs_types::p8num::P8NumStringConversionFlags;

pub fn install_pico8_base(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	p8rs_piccolo::stdlib::load_base(ctx);
	
	ctx.set_global("tostr", tostr::callback(ctx));
	ctx.set_global("tonum", tonum::callback(ctx));
	ctx.set_global("printh", printh::callback(ctx));
	ctx.set_global("print", print::callback(ctx));
}

#[api_callback]
pub fn tostr<'gc>(ctx: Context<'gc>, val: Value<'gc>, opts: Option<Value<'gc>>) -> Result<Option<Value<'gc>>, RuntimeError> {
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
pub fn tonum<'gc>(val: String, opts: Option<u8>) -> Result<Option<Value<'gc>>, RuntimeError> {
	let flags: NumberConversionFlags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
	let conversion = number_from_ascii(&val, flags);
	if conversion.is_ok() {
		Ok(Some(conversion.unwrap()))
	}else{
		Ok(None)
	}
}

#[api_callback]
pub fn printh(text: String, filename: Option<String>, _overwrite: Option<bool>, _save_to_desktop: Option<bool>) -> Result<(), RuntimeError> {
	if let Some(filename_str) = filename {
		info!("[printh/{}] {}", filename_str, text);
	} else {
		info!("[printh] {}", text);
	}
	Ok(())
}

#[api_callback]
pub fn print(text: String, _x: Option<i16>, _y: Option<i16>, _color: Option<u8>) -> Result<(), RuntimeError> {
	info!("[print] {}", text);
	// todo: implement on-screen printing
	Ok(())
}




