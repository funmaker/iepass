mod base;
mod math;
mod table;
mod memory;
mod gfx;
mod string;
mod internal;
mod print;
mod drawing;
mod input;
mod sound;
mod rnd;

use core::alloc::Allocator;
use p8rs_piccolo::Context;

pub fn install_pico8_apis(ctx: Context) {
	base::install_pico8_base(ctx);
	input::install_pico8_input(ctx);
	math::install_pico8_math(ctx);
	gfx::install_pico8_gfx(ctx);
	drawing::install_pico8_drawing(ctx);
	print::install_pico8_print(ctx);
	memory::install_pico8_memory(ctx);
	string::install_pico8_string(ctx);
	table::install_pico8_table(ctx);
	sound::install_pico8_sound(ctx);
	rnd::install_pico8_rnd(ctx);
	internal::install_pico8_internal(ctx);
}
