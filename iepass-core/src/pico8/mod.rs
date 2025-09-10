#[allow(unused_imports)]
use micromath::F32Ext;
use core::alloc::Allocator;
use core::cell::{RefCell, RefMut};
use alloc::alloc::Global;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use piccolo::{Callback, CallbackReturn, Context, FromMultiValue, IntoMultiValue, Lua, IntoValue, Table, Closure, Executor, StashedExecutor, Fuel, ExecutorMode, Value, Variadic};
use piccolo::table::InvalidTableKey;

pub mod memory;
pub mod palette;
pub mod env;

use env::Env;

pub struct Pico8VM<A: Allocator = Global> {
	lua: Lua,
	env: Rc<RefCell<Env<A>>>,
	executor: Option<StashedExecutor>,
}

impl Pico8VM<Global> {
	pub fn new() -> Result<Pico8VM, InvalidTableKey> {
		Self::new_in(Global)
	}
}

impl<A: Allocator + Clone + 'static> Pico8VM<A> {
	pub fn new_in(alloc: A) -> Result<Pico8VM<A>, InvalidTableKey> {
		let mut vm = Self {
			env: Rc::new(RefCell::new(Env::new(alloc))),
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
					if !executor.step(ctx, &mut fuel).unwrap() {
						panic!("Out of fuel!");
					}
					
					match executor.mode() {
						ExecutorMode::Normal => continue,
						ExecutorMode::Stopped => break,
						ExecutorMode::Result => {
							let value = executor.take_result::<Value>(ctx);
							
							match executor.mode() {
								ExecutorMode::Suspended => executor.resume(ctx, ()).unwrap(),
								ExecutorMode::Stopped => info!("Execution stopped. {:?}", format!("{:?}", value)),
								mode => panic!("Unexpected executor mode: {}", format!("{:?}", mode)),
							}
							
							break;
						},
						mode => panic!("Unexpected executor mode: {}", format!("{:?}", mode)),
					}
				}
			});
		}
	}
	
	pub fn env(&self) -> RefMut<'_, Env<A>> {
		self.env.borrow_mut()
	}
	
	fn install_pico8_lib(&mut self) -> Result<(), InvalidTableKey> {
		self.lua.enter(|ctx: Context| {
			// General
			ctx.set_global("flip", Callback::from_fn(&ctx, |_, _, _| Ok(CallbackReturn::Yield { to_thread: None, then: None })));
			
			// Math
			ctx.set_global("abs", callback("abs", ctx, |_, v: f32| v.abs()));
			ctx.set_global("atan2", callback("atan2", ctx, |_, (dx, dy): (f32, f32)| dy.atan2(dx)));
			ctx.set_global("ceil", callback("ceil", ctx, |_, v: f32| v.ceil()));
			ctx.set_global("flr", callback("flr", ctx, |_, v: f32| v.floor()));
			ctx.set_global("min", callback("min", ctx, |_, (a, b): (f32, f32)| a.min(b)));
			ctx.set_global("max", callback("max", ctx, |_, (a, b): (f32, f32)| a.max(b)));
			ctx.set_global("mid", callback("mid", ctx, |_, (a, b, c): (f32, f32, f32)| if (a <= b) != (a <= c) { a } else if (b <= a) != (b <= c) { b } else { c }));
			ctx.set_global("sgn", callback("sgn", ctx, |_, v: f32| if v < 0f32 { -1 } else { 1 }));
			
			// Strings
			ctx.set_global("sub", callback("abs", ctx, |_, (text, start, end): (String, i32, Option<i32>)| {
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
			
			// Debug
			ctx.set_global("printh", callback("printh", ctx, |_, (text, filename, _overwrite, _save_to_desktop): (String, Option<String>, Option<bool>, Option<bool>)| {
				if let Some(filename_str) = filename {
					info!("[printh/{}] {}", filename_str, text);
				} else {
					info!("[printh] {}", text);
				}
			}));
			
			// Memory
			let env = self.env.clone();
			ctx.set_global("peek", callback("peek", ctx, move |ctx, (addr, n): (u32, Option<u32>)| {
				let env = env.borrow();
				let n = n.unwrap_or(1);
				if n == 1 { return Value::Integer(env.memory[addr as usize] as i64); }
				
				let table = Table::new(&ctx);
				for (pos, byte) in env.memory[addr as usize .. (addr + n) as usize].iter().enumerate() {
					table.set(ctx, pos as u32 + 1, byte).unwrap();
				}
				Value::Table(table)
			}));
			
			let env = self.env.clone();
			ctx.set_global("poke", callback("poke", ctx, move |_, (addr, mut bytes): (u32, Variadic<alloc::vec::Vec<u8>>)| {
				let mut env = env.borrow_mut();
				if bytes.is_empty() { bytes.push(0) }
				for (pos, byte) in bytes.into_iter().enumerate() {
					env.memory[addr as usize + pos] = byte;
				}
			}));
			
			// GFX
			let env = self.env.clone();
			ctx.set_global("camera", callback("camera", ctx, move |_, (x, y): (Option<u32>, Option<u32>)| {
				let mut env = env.borrow_mut();
				let old = (env.memory.read_u16_le(0x5f28), env.memory.read_u16_le(0x5f2a));
				env.memory.write_u16_le(0x5f28, x.unwrap_or(0) as u16);
				env.memory.write_u16_le(0x5f2a, y.unwrap_or(0) as u16);
				old
			}));
			
			let env = self.env.clone();
			ctx.set_global("color", callback("color", ctx, move |_, val: Option<u32>| {
				let mut env = env.borrow_mut();
				let old = env.memory[0x5f25];
				if let Some(val) = val { env.memory[0x5f25] = val as u8; }
				old
			}));
			
			let env = self.env.clone();
			ctx.set_global("clip", callback("clip", ctx, move |_, (x, y, w, h, clip_previous): (Option<u8>, Option<u8>, Option<u8>, Option<u8>, Option<bool>)| {
				let mut env = env.borrow_mut();
				let [x_begin_old, y_begin_old, x_end_old, y_end_old] = env.memory[0x5f20..=0x5f23].try_into().unwrap();
				
				if let Some(x) = x && let Some(y) = y && let Some(w) = w && let Some(h) = h {
					let mut x_begin = x;
					let mut y_begin = y;
					let mut x_end = x + w;
					let mut y_end = y + h;
					
					if clip_previous.unwrap_or(false) {
						if x_begin < x_begin_old { x_begin = x_begin_old; }
						if y_begin < y_begin_old { y_begin = y_begin_old; }
						if x_end > x_end_old { x_end = x_end_old; }
						if y_end > y_end_old { y_end = y_end_old; }
					}
					
					env.memory[0x5f20] = x_begin;
					env.memory[0x5f21] = y_begin;
					env.memory[0x5f22] = x_end.min(128);
					env.memory[0x5f23] = y_end.min(128);
				}else{
					env.memory[0x5f20] = 0;
					env.memory[0x5f21] = 0;
					env.memory[0x5f22] = 128;
					env.memory[0x5f23] = 128;
				}
				
				(x_begin_old, y_begin_old, x_end_old, y_end_old)
			}));
			
			
			let env = self.env.clone();
			ctx.set_global("pal", callback("pal", ctx, move |_, args: Variadic<Vec<Value>>| {
				let argc = args.len();
				assert!(argc >= 1 && argc <= 3, "Invalid number of arguments");
				
				let mut env = env.borrow_mut();
				
				if let Value::Table(t) = args[0] {
					let base = env.memory.base_addr_palette(if argc > 1 && let Value::Integer(p) = args[1] { p as u8 } else { 0 }) as usize;
					for (k, v) in t {
						if let Value::Integer(k) = k && let Value::Integer(v) = v {
							env.memory[base + (k % 16) as usize] = v as u8;
						}
					}
				}else if let Value::Integer(c0) = args[0] && let Value::Integer(c1) = args[1] {
					let base = env.memory.base_addr_palette(if argc > 2 && let Value::Integer(p) = args[2] { p as u8 } else { 0 }) as usize;
					env.memory[base + (c0 % 16) as usize] = c1 as u8;
				}else{
					panic!("Invalid arguments");
				}
			}));
			
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
	use alloc::vec::Vec;
	use piccolo::{Closure, Executor, Value, Variadic};
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
		
		
		vm.lua.finish(&ex).unwrap();
		
		let res = vm.lua.try_enter(|ctx| {
			let exec = ctx.fetch(&ex);
			let _vals = exec.take_result::<Variadic<Vec<Value>>>(ctx)??;
			
			// TODO: co to?
			// let statics = vals.into_iter().map(|x| {
			// 	match x {
			// 		Value::Nil         => StaticValue::Nil,
			// 		Value::Boolean(b)  => StaticValue::Boolean(b),
			// 		Value::Integer(i)  => StaticValue::Integer(i),
			// 		Value::Number(n)   => StaticValue::Number(n),
			// 		Value::String(s)   => StaticValue::from(ctx.stash(s)),
			// 		Value::Table(t)    => StaticValue::from(ctx.stash(t)),
			// 		Value::Function(f) => StaticValue::from(ctx.stash(f)),
			// 		Value::Thread(_)   => StaticValue::Nil,
			// 		Value::UserData(u) => StaticValue::from(ctx.stash(u)),
			// 	}
			// }).collect::<Vec<StaticValue>>();
			//
			// Ok(statics)
			
			Ok(())
		}).unwrap();
		
		info!("XDD {:?}", res);
	}
}