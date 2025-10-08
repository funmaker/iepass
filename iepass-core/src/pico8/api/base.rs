use alloc::string::String;
use piccolo::{Callback, CallbackReturn, Context, Value};
use crate::pico8::api;
use super::callback;
use crate::pico8::numeric::{number_from_string, NumberConversionFlags};

pub fn install_pico8_base(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	piccolo::stdlib::load_base(ctx);
	
	// todo: format flags
	ctx.set_global("tostr", ctx.get_global::<Value>("tostring").unwrap());
	
	ctx.set_global("tonum", callback("tonum", ctx, move |_, (val, opts): (String, Option<u8>)| {
		let flags: NumberConversionFlags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
		let conversion = number_from_string(val.as_str(), flags);
		if conversion.is_ok() {
			Some(conversion.unwrap())
		}else{
			None
		}
	}));
	
	ctx.set_global("flip", Callback::from_fn(&ctx, |_, _, _| Ok(CallbackReturn::Yield { to_thread: None, then: None })));
	
	
	ctx.set_global("printh", api::callback("printh", ctx, |_, (text, filename, _overwrite, _save_to_desktop): (String, Option<String>, Option<bool>, Option<bool>)| {
		if let Some(filename_str) = filename {
			info!("[printh/{}] {}", filename_str, text);
		} else {
			info!("[printh] {}", text);
		}
	}));
}

