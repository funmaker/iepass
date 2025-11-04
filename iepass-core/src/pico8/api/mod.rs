mod base;
mod math;
mod table;
mod memory;
mod gfx;
mod string;
mod internal;
mod print;

use core::alloc::Allocator;
use p8rs_piccolo::Context;

pub fn install_pico8_apis<A: Allocator + 'static>(ctx: Context) {
	base::install_pico8_base(ctx);
	math::install_pico8_math(ctx);
	gfx::install_pico8_gfx::<A>(ctx);
	print::install_pico8_print::<A>(ctx);
	memory::install_pico8_memory::<A>(ctx);
	string::install_pico8_string(ctx);
	table::install_pico8_table(ctx);
	internal::install_pico8_internal::<A>(ctx);
}
