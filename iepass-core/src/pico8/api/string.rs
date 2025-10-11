use alloc::borrow::ToOwned;
use alloc::string::String;
use p8rs_piccolo::{Context, RuntimeError};
use super::{set_global_callback_simple};

pub fn install_pico8_string(ctx: Context) {
	set_global_callback_simple("sub", ctx, sub);
}

fn get_string_offset(len: usize, offset: i32) -> usize {
	match offset {
		..0 => len - ((-offset-1) as usize).min(len),
		1.. => (offset as usize - 1).min(len),
		0 => 0,
	}
}

pub fn sub((text, start, end): (String, i32, Option<i32>)) -> Result<String, RuntimeError> {
	let len = text.len();
	let start = get_string_offset(len, start);
	let end = end.map(|e| get_string_offset(len, e)).unwrap_or(len);
	if end <= start {
		Ok("".to_owned())
	} else {
		Ok(text[start..end].to_owned())
	}
}
