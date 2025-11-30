use alloc::format;
use core::alloc::Allocator;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Callback, CallbackReturn, Context, Execution, IntoValue, String};

use crate::vm::Runtime;

pub fn install_pico8_internal(ctx: Context) {
	ctx.set_global("_set_fps", _set_fps::callback(ctx));
	
	ctx.set_global("flip", Callback::from_fn(&ctx, |_, _, _, _| Ok(CallbackReturn::Yield { to_thread: None, then: None })));
	ctx.set_global("stop", Callback::from_fn(&ctx, move |ctx, mut exec: Execution, mut stack, _| {
		let (message, _x, _y, _col): (Option<String>, Option<u16>, Option<u16>, Option<u8>) = stack.consume(ctx)
		                .map_err(|err| format!("[stop]: {err}").into_value(ctx))?;
		
		if let Some(_message) = message {
			// todo: x, y, col
			// print(message, Some(String::from_slice(&ctx, b"stop()")), None, None)?;
		}
		
		stack.clear();
		
		exec.fuel().interrupt();
		Ok(CallbackReturn::Yield { to_thread: None, then: None })
	}));
}

#[api_callback]
pub fn _set_fps(rt: &mut Runtime, new_fps: i16) -> i16 {
	let old_fps = rt.target_fps;
	if new_fps <= 0 {
		rt.target_fps = 30;
	} else if new_fps > 1000 {
		rt.target_fps = 1000;
	} else {
		rt.target_fps = new_fps.cast_unsigned();
	}
	
	old_fps.cast_signed()
}
