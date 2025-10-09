use alloc::alloc::Global;
use alloc::format;
use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cell::{RefCell, RefMut};
use p8rs_piccolo::table::InvalidTableKey;
use p8rs_piccolo::{Closure, Context, Executor, ExecutorMode, Fuel, Lua, StashedExecutor, Value};

pub mod memory;
pub mod palette;
pub mod font;
pub mod env;
mod numeric;
mod api;

use crate::pico8::api::install_pico8_apis;
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
			let env = self.env.clone();
			install_pico8_apis(env, ctx);
			
			Ok(())
		})?;
		
		
		Ok(())
	}
}


#[cfg(test)]
mod test {
	use crate::pico8::Pico8VM;
	use alloc::vec::Vec;
	use p8rs_piccolo::{Closure, Executor, Value, Variadic};
	
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