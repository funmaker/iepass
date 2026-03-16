use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Execution, Function, Value};
use p8rs_types::p8num::P8Num;
use crate::vm::api::print::print;
use crate::vm::Runtime;

pub fn load(ctx: Context) {
	ctx.set_global(b"set_draw_slice", set_draw_slice::callback(ctx));
	ctx.set_global(b"stop", stop::callback(ctx));
	ctx.set_global(b"flip", flip::callback(ctx));
	ctx.set_global(b"extcmd", extcmd::callback(ctx));
	ctx.set_global(b"holdframe", holdframe::callback(ctx));
	ctx.set_global(b"_startframe", _startframe::callback(ctx));
	ctx.set_global(b"_get_menu_item_selected", _get_menu_item_selected::callback(ctx));
	ctx.set_global(b"_update_buttons", _update_buttons::callback(ctx));
	ctx.set_global(b"_update_framerate", _update_framerate::callback(ctx));
	ctx.set_global(b"_set_mainloop_exists", _set_mainloop_exists::callback(ctx));
	ctx.set_global(b"_set_fps", _set_fps::callback(ctx));
	ctx.set_global(b"_mark_cpu", _mark_cpu::callback(ctx));
	ctx.set_global(b"_menuitem", _menuitem::callback(ctx));
	ctx.set_global(b"_map_display", _map_display::callback(ctx));
	ctx.set_global(b"__flipped", __flipped::callback(ctx));
	ctx.set_global(b"__dbg", __dbg::callback(ctx));
}

#[api_callback]
pub fn set_draw_slice() {}

#[api_callback]
pub fn stop<'gc>(ctx: Context<'gc>, rt: &mut Runtime, mut exec: Execution<'gc, '_>, message: Option<Value<'gc>>, x: Option<P8Num>, y: Option<P8Num>, col: Option<P8Num>) {
	if message.is_some() {
		print(ctx, rt, message, x, y, col);
	}
	
	rt.stopped = true;
	exec.fuel().interrupt();
}

#[api_callback]
pub fn flip<'gc>(mut exec: Execution<'gc, '_>, rt: &mut Runtime) {
	rt.holdframe = false;
	exec.fuel().interrupt();
}

#[api_callback]
pub fn extcmd() {}

#[api_callback]
pub fn holdframe(rt: &mut Runtime) {
	rt.holdframe = true;
}

pub use holdframe as _startframe;

#[api_callback]
pub fn _get_menu_item_selected() {}

#[api_callback]
pub fn _update_buttons(rt: &mut Runtime) {
	rt.update_buttons();
}

#[api_callback]
pub fn _update_framerate() {}

#[api_callback]
pub fn _set_mainloop_exists() {}

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

#[api_callback]
pub fn _mark_cpu() {}

#[api_callback]
pub fn _menuitem() {}

#[api_callback]
pub fn _map_display() -> bool {
	true
}

#[api_callback]
pub fn __flipped() {}

#[api_callback]
pub fn __dbg(fun: Function) {
	match fun {
		Function::Closure(cl) => println!("__dbg: {:#?}", cl.prototype().opcodes),
		Function::Callback(_) => println!("<callback>"),
	}
}
