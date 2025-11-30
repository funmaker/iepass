use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Table, Stack, Value};

pub fn install_pico8_table(ctx: Context) {
	ctx.set_global("pack", pack::callback(ctx));
	ctx.set_global("unpack", unpack::callback(ctx));
	ctx.set_global("count", count::callback(ctx));
}

#[api_callback]
fn pack<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>) {
	let t = Table::new(&ctx);
	for i in 0..stack.len() {
		t.set(ctx, (i as i16).wrapping_add(1), stack[i]).unwrap();
	}
	t.set(ctx, "n", stack.len() as i16).unwrap();
	stack.replace(ctx, t);
}

#[api_callback]
fn unpack<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Table<'gc>, start: Option<i16>, end: Option<i16>) {
	let start = start.unwrap_or(1);
	let end = end.unwrap_or_else(|| table.length().cast_signed());
	
	if start <= end {
		stack.resize((end - start + 1) as usize);
		for i in start..=end {
			stack[(i - start) as usize] = table.get_value(ctx, i);
		}
	}
}

#[api_callback]
pub fn count<'gc>(_ctx: Context<'gc>, table: Table<'gc>, value: Option<Value<'gc>>) -> i16 {
	if let Some(_value) = value {
		let count = 0;
		for _i in 1..=table.length() {
			// TODO: move to kernel for ez equality?
		}
		count
	} else {
		table.length().cast_signed()
	}
}