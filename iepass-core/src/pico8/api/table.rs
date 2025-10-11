use p8rs_piccolo::{Callback, CallbackReturn, Context, Table};

pub fn install_pico8_table(ctx: Context) {
	
	ctx.set_global("pack", Callback::from_fn(&ctx, |ctx, _, mut stack| {
		let t = Table::new(&ctx);
		for i in 0..stack.len() {
			t.set(ctx, (i as i16).wrapping_add(1), stack[i]).unwrap();
		}
		t.set(ctx, "n", stack.len() as i16).unwrap();
		stack.replace(ctx, t);
		Ok(CallbackReturn::Return)
	}));
	
	ctx.set_global("unpack", Callback::from_fn(&ctx, |ctx, _, mut stack| {
		let (table, start, end): (Table, Option<i16>, Option<i16>) =
			stack.consume(ctx)?;
		let start = start.unwrap_or(1);
		let end = end.unwrap_or_else(|| table.length().cast_signed());
		
		if start <= end {
			stack.resize((end - start + 1) as usize);
			for i in start..=end {
				stack[(i - start) as usize] = table.get_value(ctx, i);
			}
		}
		
		Ok(CallbackReturn::Return)
	}));
}