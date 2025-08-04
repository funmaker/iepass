use piccolo::{Callback, CallbackReturn, Context, Closure, Executor, FromMultiValue, FromValue, Function, IntoMultiValue, Lua, IntoValue};

struct Pico8VM {
	memory: [u8; 0x10000],
	lua: Box<Lua>,
}

impl Pico8VM {
	fn new() -> Pico8VM {
		let mut lua = Box::new(Lua::empty());
		
		let mut vm = Self {
			memory: [0; 0x10000],
			lua,
		};
		
		vm.install_pico8_lib();
		
		vm
	}
	
	fn install_pico8_lib(&mut self) {
		fn callback<'gc, F, A, R>(name: &'static str, mc: Context<'gc>, f: F) -> Callback<'gc>
		where
			F: Fn(Context<'gc>, A) -> Option<R> + 'static,
			A: FromMultiValue<'gc>,
			R: IntoMultiValue<'gc>,
		{
			Callback::from_fn(&mc, move |ctx, _, mut stack| {
				if let Some(res) = f(ctx, stack.consume(ctx)?) {
					stack.replace(ctx, res);
					Ok(CallbackReturn::Return)
				} else {
					Err(format!("Bad argument to {name}").into_value(ctx).into())
				}
			})
		}
		
		self.lua.enter(|ctx| {
			
			// Math
			
			ctx.set_global("abs", callback("abs", ctx, |_, v: f32| { Some(v.abs()) })).unwrap();
			
			ctx.set_global("atan2", callback("atan2", ctx, |_, (dx, dy): (f32, f32)| { Some(dy.atan2(dx)) })).unwrap();
			
			ctx.set_global("ceil", callback("ceil", ctx, |_, v: f32| { Some(v.ceil()) })).unwrap();
			
			ctx.set_global("flr", callback("flr", ctx, |_, v: f32| { Some(v.floor()) })).unwrap();
			
			ctx.set_global("min", callback("min", ctx, |_, (a, b): (f32, f32)| { Some(a.min(b)) })).unwrap();
			
			ctx.set_global("max", callback("max", ctx, |_, (a, b): (f32, f32)| { Some(a.max(b)) })).unwrap();
			
			ctx.set_global("mid", callback("mid", ctx, |_, (a, b, c): (f32, f32, f32)| {
				// Some(a + b + c - a.min(b).min(c) - a.max(b).max(c))
				Some(if (a <= b) != (a <= c) { a } else if (b <= a) != (b <= c) { b } else { c })
			})).unwrap();
			
			ctx.set_global("sgn", callback("sgn", ctx, |_, v: f32| { Some(if v < 0f32 { -1 } else { 1 }) })).unwrap();
			
			// Debug
			
			ctx.set_global("printh", callback("printh", ctx, |_, (text, filename, overwrite, save_to_desktop): (String, Option<String>, Option<bool>, Option<bool>)| {
				if let Some(filename_str) = filename {
					println!("[printh/{}] {}", filename_str, text);
				}else{
					println!("[printh] {}", text);
				}
				
				Some(())
			})).unwrap();
			
		});
	}
}

#[cfg(test)]
mod test {
	use piccolo::{Callback, CallbackReturn, Closure, Executor, FromValue, Function, Lua, StaticValue, Value, Variadic};
	use piccolo::registry::Fetchable;
	use crate::pico8::Pico8VM;
	
	#[test]
	pub fn it_works() {
		let source = b"
			printh(\"test from lua!\")
			printh(\"\" .. mid(5, 10, 15), \"mid\")
			return sgn(0)
        ";
		
		let mut vm = Pico8VM::new();
		
		let ex = vm.lua.try_enter(|ctx| {
			let env = ctx.globals();
			let closure = Closure::load_with_env(ctx, None, &source[..], env)?;
			let ex = Executor::start(ctx, closure.into(), ());
			
			Ok(ctx.stash(ex))
		}).unwrap();
		
		
		vm.lua.finish(&ex);
		
		let res = vm.lua.try_enter(|ctx| {
			let exec = ctx.fetch(&ex);
			let vals = exec.take_result::<Variadic<Vec<Value>>>(ctx)??;
			
			let statics = vals.into_iter().map(|x| {
				match x {
					Value::Nil => StaticValue::Nil,
					Value::Boolean(b) => StaticValue::Boolean(b),
					Value::Integer(i) => StaticValue::Integer(i),
					Value::Number(n)  => StaticValue::Number(n),
					Value::String(s)  => StaticValue::from(ctx.stash(s)),
					Value::Table(t)   => StaticValue::from(ctx.stash(t)),
					Value::Function(f)=> StaticValue::from(ctx.stash(f)),
					Value::Thread(th) => StaticValue::Nil,
					Value::UserData(u)=> StaticValue::from(ctx.stash(u)),
				}
			}).collect::<Vec<StaticValue>>();
			
			Ok(statics)
		}).unwrap();
		
		println!("XDD {:?}", res);
	}
}