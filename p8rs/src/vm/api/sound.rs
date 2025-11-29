use p8rs_macros::api_callback;
use p8rs_piccolo::Context;

pub fn install_pico8_sound(ctx: Context) {
	ctx.set_global("music", music::callback(ctx));
	ctx.set_global("sfx", sfx::callback(ctx));
}

#[api_callback]
pub fn music(_n: Option<i16>, _fadems: Option<i16>, _channelmask: Option<i16>) {
	
}

#[api_callback]
pub fn sfx(_n: Option<i16>, _channel: Option<i16>, _offset: Option<i16>, _length: Option<i16>) {
	
}
