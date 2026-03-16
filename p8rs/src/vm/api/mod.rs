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

use p8rs_piccolo::Context;

pub fn load_all(ctx: Context) {
	base::load(ctx);
	internal::load(ctx);
	consts::load(ctx);
	math::load(ctx);
	rnd::load(ctx);
	string::load(ctx);
	table::load(ctx);
	coroutine::load(ctx);
	memory::load(ctx);
	input::load(ctx);
	gfx::load(ctx);
	drawing::load(ctx);
	print::load(ctx);
	sound::load(ctx);
}
