use alloc::borrow::ToOwned;
use alloc::string::String;
use piccolo::Context;
use super::callback;

pub fn install_pico8_string(ctx: Context) {
	
	ctx.set_global("sub", callback("sub", ctx, |_, (text, start, end): (String, i32, Option<i32>)| {
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
			"".to_owned()
		} else {
			text[start..end].to_owned()
		}
	}));
	
}