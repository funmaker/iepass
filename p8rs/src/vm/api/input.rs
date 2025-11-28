use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Value};
use crate::vm::Runtime;

pub fn install_pico8_input(ctx: Context) {
	ctx.set_global("btn", btn::callback(ctx));
	ctx.set_global("btnp", btnp::callback(ctx));
}

#[api_callback]
pub fn btn<'gc>(rt: &mut Runtime, i: Option<i16>, p: Option<i16>) -> Value<'gc> {
	if let Some(btn) = i {
		let player = p.unwrap_or(0);
		if player >= 0 && player < 8 {
			rt.buttons.button_pressed(player as usize, btn as usize).into()
		} else {
			false.into()
		}
	} else {
		(rt.buttons.buttons_pressed(0) as u16 | (rt.buttons.buttons_pressed(1) as u16) << 8).cast_signed().into()
	}
}

#[api_callback]
pub fn btnp<'gc>(rt: &mut Runtime, i: Option<i16>, p: Option<i16>) -> Value<'gc> {
	if let Some(btn) = i {
		let player = p.unwrap_or(0);
		if player >= 0 && player < 8 {
			rt.buttons.button_pressed_now(player as usize, btn as usize).into()
		} else {
			false.into()
		}
	} else {
		(rt.buttons.buttons_pressed_now(0) as u16 | (rt.buttons.buttons_pressed_now(1) as u16) << 8).cast_signed().into()
	}
}
