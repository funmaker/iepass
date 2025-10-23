use super::set_global_callback_simple;
use crate::pico8::numeric::{number_from_string, NumberConversionFlags};
use alloc::string::String;
use p8rs_piccolo::{Context, RuntimeError, Value};

pub fn install_pico8_base(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	p8rs_piccolo::stdlib::load_base(ctx);
	
	// todo: format flags
	ctx.set_global("tostr", ctx.get_global::<Value>("tostring").unwrap());
	
	set_global_callback_simple("tonum", ctx, tonum);
	set_global_callback_simple("printh", ctx, printh);
	set_global_callback_simple("print", ctx, print);
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




