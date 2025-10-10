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

pub struct Pico8VMRunResult {
	pub requested_fps: u16,
	pub stopped: bool,
}

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
	
	// Returns the FPS requested - should be 30 or 60
	pub fn run(&mut self) -> Pico8VMRunResult {
		let mut stopped = false;
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
						ExecutorMode::Stopped => {
							stopped = true;
							break
						},
						ExecutorMode::Result => {
							let value = executor.take_result::<Value>(ctx);
							
							match executor.mode() {
								ExecutorMode::Suspended => executor.resume(ctx, ()).unwrap(),
								ExecutorMode::Stopped => info!("Execution stopped. {:?}", value),
								mode => panic!("Unexpected executor mode: {}", format!("{:?}", mode)),
							}
							
							break;
						},
						mode => panic!("Unexpected executor mode: {}", format!("{:?}", mode)),
					}
				}
			});
		}
		
		Pico8VMRunResult {
			requested_fps: self.env.borrow().fps,
			stopped,
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


#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum StaticValue {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f32),
	String,
	Table,
	Function,
	Thread,
	UserData,
}
// todo: context for strings / tables / maybe function names
pub fn to_static_value(x: &Value) -> StaticValue {
	match x {
		Value::Nil         => StaticValue::Nil,
		Value::Boolean(b)  => StaticValue::Boolean(b.clone()),
		Value::Integer(i)  => StaticValue::Integer(i.clone()),
		Value::Number(n)   => StaticValue::Number(n.clone() as f32),
		Value::String(_s)   => StaticValue::String,
		Value::Table(_t)    => StaticValue::Table,
		Value::Function(_f) => StaticValue::Function,
		Value::Thread(_)   => StaticValue::Thread,
		Value::UserData(_u) => StaticValue::UserData,
	}
}


#[cfg(test)]
mod test {
	use crate::pico8::{to_static_value, Pico8VM, StaticValue};
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
		
		error!("Test!"); // todo: show test output
		vm.lua.finish(&ex).unwrap();
		
		let res = vm.lua.try_enter(|ctx| {
			let exec = ctx.fetch(&ex);
			let vals = exec.take_result::<Variadic<Vec<Value>>>(ctx)??;
			
			let results = vals.iter().map(to_static_value).collect::<Vec<_>>();
			
			
			Ok(results)
		}).unwrap();
		
		assert_eq!(res.len(), 1, "expected one result");
		let first = res[0];
		if let StaticValue::Integer(i) = first {
			assert_eq!(i, 1, "first element should be 1");
		}
		
		info!("XDD {:?}", res);
	}
}
