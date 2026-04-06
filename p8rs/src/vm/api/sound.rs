use p8rs_macros::api_callback;
use p8rs_piccolo::Context;
use crate::utils::once;

pub fn install(ctx: Context) {
	ctx.set_global(b"music", music::callback(ctx));
	ctx.set_global(b"sfx", sfx::callback(ctx));
}

#[api_callback]
pub fn music(_n: Option<i16>, _fadems: Option<i16>, _channelmask: Option<i16>) {
	once!{ warn!("music is not implemented yet!"); }
}

#[api_callback]
pub fn sfx(_n: Option<i16>, _channel: Option<i16>, _offset: Option<i16>, _length: Option<i16>) {
	once!{ warn!("sfx is not implemented yet!"); }
}
