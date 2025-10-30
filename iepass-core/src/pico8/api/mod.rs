mod base;
mod math;
mod table;
mod memory;
mod gfx;
mod string;
mod internal;

use alloc::format;
use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::RefCell;
use p8rs_piccolo::{Callback, CallbackReturn, Context, FromMultiValue, IntoMultiValue, IntoValue, RuntimeError};
use crate::pico8::env::Env;

pub type EnvHandle<A> = Rc<RefCell<Env<A>>>;

pub fn install_pico8_apis<A: Allocator + Clone + 'static>(env: EnvHandle<A>, ctx: Context)
{
	base::install_pico8_base(ctx);
	math::install_pico8_math(ctx);
	gfx::install_pico8_gfx(env.clone(), ctx);
	math::install_pico8_math(ctx);
	memory::install_pico8_memory(env.clone(), ctx);
	string::install_pico8_string(ctx);
	table::install_pico8_table(ctx);
	internal::install_pico8_internal(env, ctx);
}

fn set_global_callback_ctx<'gc, F, A, R>(name: &'static str, ctx: Context<'gc>, f: F)
where F: Fn(Context<'gc>, A) -> Result<R, RuntimeError> + 'static,
      A: FromMultiValue<'gc>,
      R: IntoMultiValue<'gc>,
{
	let callback = Callback::from_fn(&ctx, move |ctx, _, mut stack, _| {
		let args = stack.consume(ctx)
		                .map_err(|err| format!("[{name}]: {err}").into_value(ctx))?;
		let ret = f(ctx, args)?;
		stack.replace(ctx, ret);
		Ok(CallbackReturn::Return)
	});
	ctx.set_global(name, callback);
}

// pub fn set_global_callback_ctx_env<'gc, F, A, R, Al>(name: &'static str, ctx: Context<'gc>, env: EnvHandle<Al>, f: F)
// where F: Fn(Context<'gc>, EnvHandle<Al>, A) -> Result<R, RuntimeError> + 'static,
//       A: FromMultiValue<'gc>,
//       R: IntoMultiValue<'gc>,
//       Al: Allocator + Clone + 'static
// {
// 	set_global_callback_ctx(name, ctx, move |ctx, args| { f(ctx, env.clone(), args) });
// }

pub fn set_global_callback_env<'gc, F, A, R, Al>(name: &'static str, ctx: Context<'gc>, env: EnvHandle<Al>, f: F)
where F: Fn(EnvHandle<Al>, A) -> Result<R, RuntimeError> + 'static,
      A: FromMultiValue<'gc>,
      R: IntoMultiValue<'gc>,
      Al: Allocator + Clone + 'static
{
	set_global_callback_ctx(name, ctx, move |_, args| { f(env.clone(), args) });
}

pub fn set_global_callback_simple<'gc, F, A, R>(name: &'static str, ctx: Context<'gc>, f: F)
where F: Fn(A) -> Result<R, RuntimeError> + 'static,
      A: FromMultiValue<'gc>,
      R: IntoMultiValue<'gc>,
{
	set_global_callback_ctx(name, ctx, move |_, args| { f(args) });
}