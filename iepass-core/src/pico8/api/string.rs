use alloc::borrow::ToOwned;
use alloc::string::String;
use piccolo::{Context, RuntimeError};
use super::{set_global_callback_simple};

pub fn install_pico8_string(ctx: Context) {
	set_global_callback_simple("sub", ctx, sub);
}

pub fn sub((text, start, end): (String, i32, Option<i32>)) -> Result<String, RuntimeError> {
	let start = match start {
		..0 => text.len() - ((-start-1) as usize).min(text.len()),
		1.. => (start as usize - 1).min(text.len()),
		0 => 0,
	};
	let end = end.unwrap_or(-1);
	let end = match end {
		..0 => text.len() - ((-end-1) as usize).min(text.len()),
		1.. => (end as usize - 1).min(text.len()),
		0 => 0,
	};
	if end <= start {
		Ok("".to_owned())
	} else {
		Ok(text[start..end].to_owned())
	}
}
