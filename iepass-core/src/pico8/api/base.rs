use super::set_global_callback_simple;
use crate::pico8::numeric::{number_from_string, NumberConversionFlags};
use alloc::string::String;
use piccolo::{Callback, CallbackReturn, Context, RuntimeError, Value};

pub fn install_pico8_base(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	piccolo::stdlib::load_base(ctx);
	
	// todo: format flags
	ctx.set_global("tostr", ctx.get_global::<Value>("tostring").unwrap());
	
	ctx.set_global("flip", Callback::from_fn(&ctx, |_, _, _| Ok(CallbackReturn::Yield { to_thread: None, then: None })));
	
	set_global_callback_simple("tonum", ctx, tonum);
	set_global_callback_simple("printh", ctx, printh);
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




