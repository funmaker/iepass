pub mod base;
pub mod internal;
pub mod consts;
pub mod math;
pub mod rnd;
pub mod string;
pub mod table;
pub mod coroutine;
pub mod memory;
pub mod input;
pub mod gfx;
pub mod drawing;
pub mod print;
pub mod sound;
pub mod cartdata;
pub mod cmd;

use p8rs_piccolo::Context;

pub fn load_all(ctx: Context) {
	base::install(ctx);
	internal::install(ctx);
	consts::install(ctx);
	math::install(ctx);
	rnd::install(ctx);
	string::install(ctx);
	table::install(ctx);
	coroutine::install(ctx);
	memory::install(ctx);
	input::install(ctx);
	gfx::install(ctx);
	drawing::install(ctx);
	print::install(ctx);
	sound::install(ctx);
	cartdata::install(ctx);
	cmd::install(ctx);
}
