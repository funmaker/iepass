use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use piccolo::{Callback, CallbackReturn, Context, FromMultiValue, IntoMultiValue, Lua, IntoValue, InvalidTableKey, Table, Closure, Executor, StashedExecutor, Fuel, ExecutorMode, Value, Variadic};

pub mod memory;
pub mod palette;

use memory::Memory;

pub struct Pico8VM {
	memory: Rc<RefCell<Memory>>,
	lua: Lua,
	executor: Option<StashedExecutor>,
}

impl Pico8VM {
	pub fn new() -> Result<Pico8VM, InvalidTableKey> {
		let mut vm = Self {
			memory: Rc::new(RefCell::new(Memory::new())),
			lua: Lua::empty(),
			executor: None,
		};
		
		vm.install_pico8_lib()?;
		
		Ok(vm)
	}
	
	pub fn load(&mut self, source: &[u8]) {
		let ex = self.lua.try_enter(|ctx| {
			let env = ctx.globals();
			let closure = Closure::load_with_env(ctx, None, source, env)?;
			let ex = Executor::start(ctx, closure.into(), ());
			
			Ok(ctx.stash(ex))
		}).unwrap();
		
		self.executor = Some(ex);
	}
	
	pub fn run(&mut self) {
		if let Some(executor) = self.executor.as_mut() {
			let mut fuel = Fuel::with(10240);
			self.lua.enter(|ctx| {
				let executor = ctx.fetch(executor);
				
				loop {
					if !executor.step(ctx, &mut fuel) {
						panic!("Out of fuel!");
					}
					
					match executor.mode() {
						ExecutorMode::Normal => continue,
						ExecutorMode::Stopped => break,
						ExecutorMode::Result => {
							let value = executor.take_result::<Value>(ctx);
							
							match executor.mode() {
								ExecutorMode::Suspended => executor.resume(ctx, ()).unwrap(),
								ExecutorMode::Stopped => println!("{value:?}"),
								mode => panic!("Unexpected executor mode: {mode:?}"),
							}
							
							break;
						},
						mode => panic!("Unexpected executor mode: {mode:?}"),
					}
				}
			});
		}
	}
	
	pub fn memory(&self) -> RefMut<'_, Memory> {
		self.memory.borrow_mut()
	}
	
	fn install_pico8_lib(&mut self) -> Result<(), InvalidTableKey> {
		self.lua.enter(|ctx| {
			// General
			ctx.set_global("flip", Callback::from_fn(&ctx, |_, _, _| Ok(CallbackReturn::Yield { to_thread: None, then: None })))?;
			
			// Math
			ctx.set_global("abs", callback("abs", ctx, |_, v: f32| v.abs()))?;
			ctx.set_global("atan2", callback("atan2", ctx, |_, (dx, dy): (f32, f32)| dy.atan2(dx)))?;
			ctx.set_global("ceil", callback("ceil", ctx, |_, v: f32| v.ceil()))?;
			ctx.set_global("flr", callback("flr", ctx, |_, v: f32| v.floor()))?;
			ctx.set_global("min", callback("min", ctx, |_, (a, b): (f32, f32)| a.min(b)))?;
			ctx.set_global("max", callback("max", ctx, |_, (a, b): (f32, f32)| a.max(b)))?;
			ctx.set_global("mid", callback("mid", ctx, |_, (a, b, c): (f32, f32, f32)| if (a <= b) != (a <= c) { a } else if (b <= a) != (b <= c) { b } else { c }))?;
			ctx.set_global("sgn", callback("sgn", ctx, |_, v: f32| if v < 0f32 { -1 } else { 1 }))?;
			
			// Debug
			ctx.set_global("printh", callback("printh", ctx, |_, (text, filename, _overwrite, _save_to_desktop): (String, Option<String>, Option<bool>, Option<bool>)| {
				if let Some(filename_str) = filename {
					println!("[printh/{}] {}", filename_str, text);
				} else {
					println!("[printh] {}", text);
				}
			}))?;
			
			// Memory
			let memory = self.memory.clone();
			ctx.set_global("peek", callback("peek", ctx, move |ctx, (addr, n): (u32, Option<u32>)| {
				let memory = memory.borrow();
				let table = Table::new(&ctx);
				for (pos, byte) in memory[addr as usize .. (addr + n.unwrap_or(1)) as usize].iter().enumerate() {
					table.set(ctx, pos as u32 + 1, byte).unwrap();
				}
				table
			}))?;
			
			let memory = self.memory.clone();
			ctx.set_global("poke", callback("poke", ctx, move |_, (addr, mut bytes): (u32, Variadic<Vec<u8>>)| {
				let mut memory = memory.borrow_mut();
				if bytes.is_empty() { bytes.push(0) }
				for (pos, byte) in bytes.into_iter().enumerate() {
					memory[addr as usize + pos] = byte;
				}
			}))?;
			
			Ok(())
		})?;
		
		Ok(())
	}
}

fn callback<'gc, F, A, R>(name: &'static str, ctx: Context<'gc>, f: F) -> Callback<'gc>
where F: Fn(Context<'gc>, A) -> R + 'static,
      A: FromMultiValue<'gc>,
      R: IntoMultiValue<'gc> {
	Callback::from_fn(&ctx, move |ctx, _, mut stack| {
		let args = stack.consume(ctx)
		                .map_err(|err| format!("[{name}]: {err}").into_value(ctx))?;
		let ret = f(ctx, args);
		stack.replace(ctx, ret);
		Ok(CallbackReturn::Return)
	})
}

#[cfg(test)]
mod test {
	use piccolo::{Closure, Executor, StaticValue, Value, Variadic};
	use crate::pico8::Pico8VM;
	
	#[test]
	pub fn it_works() {
		let source = b"
			printh(\"test from lua!\")
			printh(\"\" .. mid(5, 10, 15), \"mid\")
			return sgn(0)
		";
		
		let mut vm = Pico8VM::new().unwrap();
		
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
					Value::Thread(_) => StaticValue::Nil,
					Value::UserData(u)=> StaticValue::from(ctx.stash(u)),
				}
			}).collect::<Vec<StaticValue>>();
			
			Ok(statics)
		}).unwrap();
		
		println!("XDD {:?}", res);
	}
}