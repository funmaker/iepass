use p8rs_macros::api_callback;
use p8rs_piccolo::{String, Context, Stack, Value};
use crate::utils::once;

pub fn install(ctx: Context) {
	ctx.set_global(b"sub", sub::callback(ctx));
	ctx.set_global(b"ord", ord::callback(ctx));
	ctx.set_global(b"chr", chr::callback(ctx));
	ctx.set_global(b"split", split::callback(ctx));
}

#[api_callback]
pub fn sub<'gc>(ctx: Context<'gc>, str: String, start: i16, end: Option<i16>) -> String<'gc> {
	let len = str.len();
	let start = get_string_offset(len, start);
	let end = end.map(|e| get_string_offset(len, e)).unwrap_or(len - 1);
	if end < start {
		String::from_static(&ctx, &[])
	} else {
		String::from_slice(&ctx, &str[start..=end])
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

#[api_callback]
pub fn chr<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>) -> Option<String<'gc>> {
	let mut output = Vec::with_capacity(stack.len());
	
	for value in stack.drain(..) {
		if let Some(number) = value.to_number() {
			output.push(number.to_integer() as u8)
		} else {
			return None;
		}
	}
	
	Some(String::from_buffer(&ctx, output.into_boxed_slice()))
}

#[api_callback]
pub fn split() {
	once!{ warn!("split is not implemented yet!"); }
}

fn get_string_offset(len: usize, offset: i16) -> usize {
	match offset {
		..0 => len - ((-offset) as usize).min(len),
		1.. => (offset as usize - 1).min(len - 1),
		0 => 0,
	}
}
