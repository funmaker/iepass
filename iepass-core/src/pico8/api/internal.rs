use alloc::format;
use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::RefCell;
use super::{set_global_callback_env, EnvHandle};
use p8rs_piccolo::{Callback, CallbackReturn, Context, Execution, IntoValue, RuntimeError};

#[allow(unused_imports)]
use micromath::F32Ext;
use crate::pico8::api::base::printh;
use crate::pico8::env::Env;

pub fn install_pico8_internal<A: Allocator + Clone + 'static>(env: Rc<RefCell<Env<A>>>, ctx: Context) {
	set_global_callback_env("_set_fps", ctx, env.clone(), _set_fps);
	
	ctx.set_global("__flip", Callback::from_fn(&ctx, |_, _, _| Ok(CallbackReturn::Yield { to_thread: None, then: None })));
	ctx.set_global("__stop", Callback::from_fn(&ctx, move |ctx, mut exec: Execution, mut stack| {
		let (message, _x, _y, _col): (Option<alloc::string::String>, Option<u16>, Option<u16>, Option<u8>) = stack.consume(ctx)
		                .map_err(|err| format!("[stop]: {err}").into_value(ctx))?;
		
		if let Some(message) = message {
			// todo: x, y, col
			printh((message, Some("stop()".into()), None, None))?;
		}
		
		stack.clear();
		
		exec.fuel().interrupt();
		Ok(CallbackReturn::Yield { to_thread: None, then: None })
	}));
	
	
	
}

pub fn _set_fps<A: Allocator + Clone + 'static>(env: EnvHandle<A>, new_fps: i32) -> Result<u16, RuntimeError> {
	let mut env = env.borrow_mut();
	let old_fps = env.fps;
	if new_fps <= 0 {
		env.fps = 30;
	}else if new_fps > 1000 {
		env.fps = 1000;
	}else {
		env.fps = new_fps as u16;
	}
	Ok(old_fps)
}
