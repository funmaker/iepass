mod base;
mod math;
mod table;
mod memory;
mod gfx;
mod string;

use alloc::format;
use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::RefCell;
use piccolo::{Callback, CallbackReturn, Context, FromMultiValue, IntoMultiValue, IntoValue};
use crate::pico8::env::Env;

pub fn install_pico8_apis<A: Allocator + Clone + 'static>(env: Rc<RefCell<Env<A>>>, ctx: Context)
{
	base::install_pico8_base(ctx);
	math::install_pico8_math(ctx);
	gfx::install_pico8_gfx(env.clone(), ctx);
	math::install_pico8_math(ctx);
	memory::install_pico8_memory(env, ctx);
	string::install_pico8_string(ctx);
	table::install_pico8_table(ctx);
}

pub fn callback<'gc, F, A, R>(name: &'static str, ctx: Context<'gc>, f: F) -> Callback<'gc>
where F: Fn(Context<'gc>, A) -> R + 'static,
      A: FromMultiValue<'gc>,
      R: IntoMultiValue<'gc> {
	Callback::from_fn(&ctx, move |ctx, _, mut stack| {
		let args = stack.consume(ctx)
		                .map_err(|err| format!("[{name}]: {err}").into_value(ctx))?;
		let ret = f(ctx, args);
		stack.replace(ctx, ret);
		Ok(CallbackReturn::Return)
	})
}