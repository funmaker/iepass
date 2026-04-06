use core::pin::Pin;
use gc_arena::Collect;
use p8rs_macros::api_callback;
use p8rs_piccolo::{meta_ops, BoxSequence, CallbackReturn, Context, Error, Execution, RuntimeRef, Sequence, SequencePoll, Stack, Thread, ThreadMode, Value};
use p8rs_piccolo::meta_ops::MetaCallError;

pub fn install(ctx: Context) {
	ctx.set_global(b"cocreate", cocreate::callback(ctx));
	ctx.set_global(b"coresume", coresume::callback(ctx));
	ctx.set_global(b"costatus", costatus::callback(ctx));
	ctx.set_global(b"yield", r#yield::callback(ctx));
}

#[api_callback]
pub fn cocreate<'gc>(ctx: Context<'gc>, func: Value<'gc>) -> Result<Thread<'gc>, MetaCallError> {
	let thread = Thread::new(ctx);
	thread.start_suspended(&ctx, meta_ops::call(ctx, func)?).unwrap();
	Ok(thread)
}

#[api_callback]
pub fn coresume<'gc>(ctx: Context<'gc>, thread: Thread<'gc>) -> CallbackReturn<'gc> {
	CallbackReturn::Resume {
		thread,
		then: Some(BoxSequence::new(&ctx, PCall)),
	}
}

#[api_callback]
pub fn costatus(thread: Thread) -> &'static [u8] {
	match thread.mode() {
		ThreadMode::Stopped => b"dead",
		ThreadMode::Running | ThreadMode::Waiting => b"running",
		ThreadMode::Normal => b"normal",
		ThreadMode::Result | ThreadMode::Suspended => b"suspended",
	}
}

#[api_callback]
pub fn r#yield<'gc>() -> CallbackReturn<'gc> {
	CallbackReturn::Yield {
		to_thread: None,
		then: None,
	}
}

#[derive(Collect)]
#[collect(require_static)]
pub struct PCall;

impl<'gc> Sequence<'gc> for PCall {
	fn poll(
		self: Pin<&mut Self>,
		ctx: Context<'gc>,
		_exec: Execution<'gc, '_>,
		mut stack: Stack<'gc, '_>,
		_rt: RuntimeRef<'_>,
	) -> Result<SequencePoll<'gc>, Error<'gc>> {
		stack.into_front(ctx, true);
		Ok(SequencePoll::Return)
	}
	
	fn error(
		self: Pin<&mut Self>,
		ctx: Context<'gc>,
		_exec: Execution<'gc, '_>,
		error: Error<'gc>,
		mut stack: Stack<'gc, '_>,
		_rt: RuntimeRef<'_>,
	) -> Result<SequencePoll<'gc>, Error<'gc>> {
		println!("{:?}", error);
		match error {
			Error::Lua(error) => stack.replace(ctx, (false, error.0)),
			Error::Runtime(error) => stack.replace(ctx, (false, error.to_string())),
		}
		
		Ok(SequencePoll::Return)
	}
}
