use p8rs_piccolo::{String, Context, RuntimeError};
use super::{set_global_callback_ctx};

pub fn install_pico8_string(ctx: Context) {
	set_global_callback_ctx("sub", ctx, sub);
}

fn get_string_offset(len: usize, offset: i16) -> usize {
	match offset {
		..0 => len - ((-offset) as usize).min(len),
		1.. => (offset as usize - 1).min(len - 1),
		0 => 0,
	}
}

pub fn sub<'gc>(ctx: Context<'gc>, (text, start, end): (String, i16, Option<i16>)) -> Result<String<'gc>, RuntimeError> {
	let len = text.len();
	let start = get_string_offset(len, start);
	let end = end.map(|e| get_string_offset(len, e)).unwrap_or(len - 1);
	if end < start {
		Ok(String::from_static(&ctx, &[]))
	} else {
		Ok(String::from_slice(&ctx, &text[start..=end]))
	}
}
