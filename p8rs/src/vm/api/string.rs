use p8rs_macros::api_callback;
use p8rs_piccolo::{String, Context, RuntimeError, Stack, Value};

pub fn install_pico8_string(ctx: Context) {
	ctx.set_global("sub", sub::callback(ctx));
	ctx.set_global("ord", ord::callback(ctx));
}

#[api_callback]
pub fn sub<'gc>(ctx: Context<'gc>, str: String, start: i16, end: Option<i16>) -> Result<String<'gc>, RuntimeError> {
	let len = str.len();
	let start = get_string_offset(len, start);
	let end = end.map(|e| get_string_offset(len, e)).unwrap_or(len - 1);
	if end < start {
		Ok(String::from_static(&ctx, &[]))
	} else {
		Ok(String::from_slice(&ctx, &str[start..=end]))
	}
}

#[api_callback]
pub fn ord(mut stack: Stack, str: String, index: Option<i16>, count: Option<i16>) {
	let index = index.unwrap_or(1);
	let count = count.unwrap_or(1);
	
	for i in index..index+count {
		if i <= 0 || i as usize > str.len() {
			stack.push_back(Value::Nil);
		} else {
			stack.push_back(str[i as usize - 1].into());
		}
	}
}

fn get_string_offset(len: usize, offset: i16) -> usize {
	match offset {
		..0 => len - ((-offset) as usize).min(len),
		1.. => (offset as usize - 1).min(len - 1),
		0 => 0,
	}
}
