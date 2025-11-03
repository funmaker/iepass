use alloc::alloc::Global;
use alloc::boxed::Box;
use core::alloc::Allocator;
use anyhow::anyhow;
use p8rs_piccolo::table::InvalidTableKey;
use p8rs_piccolo::{Closure, CompilerError, Context, Error, Executor, ExecutorMode, ExternError, Fuel, Lua, RuntimeError, StashedExecutor, Value};
use p8rs_types::p8num::P8Num;

pub mod memory;
pub mod palette;
pub mod font;
pub mod cart;
pub mod runtime;
pub mod callbacks;
mod numeric;
mod api;

pub use runtime::Runtime;
pub use callbacks::Callbacks;
use api::install_pico8_apis;
use cart::CartLoadError;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RunResult {
	Flip,
	Stop,
	OutOfFuel,
}

pub struct Pico8VM<A: Allocator = Global> {
	lua: Lua,
	runtime: Runtime<A>,
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
			lua: Lua::empty(),
			runtime: Runtime::new(alloc),
			executor: None,
		};
		
		vm.lua.enter(|ctx: Context| {
			install_pico8_apis::<A>(ctx);
			Ok(())
		})?;
		
		Ok(vm)
	}
}

impl<A: Allocator + 'static> Pico8VM<A> {
	pub fn load(&mut self, source: &[u8]) -> Result<(), ExternError> {
		let ex = self.lua.try_enter(|ctx| {
			let closure = Closure::load(ctx, None, source)?;
			let ex = Executor::start(ctx, closure.into(), ());
			
			Ok(ctx.stash(ex))
		})?;
		
		self.executor = Some(ex);
		
		Ok(())
	}
	
	pub fn load_cartridge(&mut self, source: &[u8]) -> Result<(), CartLoadError> {
		cart::load_cartridge(self, source)
	}
	
	pub fn set_callbacks(&mut self, callbacks: impl Callbacks + 'static) {
		self.runtime.callbacks = Box::new(callbacks);
	}
	
	pub fn run(&mut self) -> Result<RunResult, ExternError> {
		loop {
			match self.run_fuel(1024*1024)? {
				RunResult::OutOfFuel => continue,
				result => return Ok(result),
			}
		}
	}
	
	pub fn run_fuel(&mut self, mut max_fuel: i32) -> Result<RunResult, ExternError> {
		if max_fuel < 1 {
			warn!("run_fuel called with {} fuel, using 1 fuel", max_fuel);
			max_fuel = 1;
		}
		
		let mut fuel = Fuel::with(max_fuel);
		let result = if let Some(executor) = self.executor.as_mut() {
			self.lua.enter(|ctx| {
				let executor = ctx.fetch(executor);
				
				loop {
					if executor.mode() == ExecutorMode::Suspended {
						executor.resume(ctx, ()).unwrap();
					}
					
					if !executor.step(ctx, &mut fuel, &mut self.runtime).unwrap() {
						if fuel.is_interrupted() {
							trace!("[run_fuel] Execution interrupted, fuel: {:?}, executor: {:?}", fuel, executor.mode());
							return Ok(RunResult::Stop)
						} else {
							trace!("[run_fuel] Out of fuel: {:?}", fuel);
							return Ok(RunResult::OutOfFuel)
						}
					}
					
					match executor.mode() {
						ExecutorMode::Normal => {
							trace!("[run_fuel] mode Normal {:?}", fuel);
							continue
						},
						ExecutorMode::Stopped => {
							trace!("[run_fuel] mode Stopped, {:?}", fuel);
							return Ok(RunResult::Stop)
						},
						ExecutorMode::Result => {
							match executor.take_result::<Value>(ctx).unwrap() {
								Ok(value) => trace!("[run_fuel] mode Result - Value: {:?}, fuel {:?}", value, fuel),
								Err(err) => {
									match &err {
										Error::Lua(e) => error!("[run_fuel] Uncaught lua error: {:?}", e.0),
										Error::Runtime(e) => error!("[run_fuel] Uncaught runtime error: {}", e.0.root_cause()),
									}
									return Err(err.into())
								}
							}
							
							match executor.mode() {
								ExecutorMode::Suspended => {
									trace!("[run_fuel] Result -> Flip, {:?}", fuel);
									return Ok(RunResult::Flip)
								},
								ExecutorMode::Stopped => {
									trace!("[run_fuel] Result -> Stopped.");
									return Ok(RunResult::Stop)
								},
								mode => panic!("Unexpected executor mode: Result -> {:?}", mode),
							}
						},
						mode => panic!("Unexpected executor mode: {:?}", mode),
					}
				}
			})
		} else {
			Err(RuntimeError::new(anyhow!("No cartridge loaded")).into())
		};
		
		trace!("[run_fuel] Step finished {:?}", result);
		
		result
	}
	
	pub fn runtime(&mut self) -> &mut Runtime<A> {
		&mut self.runtime
	}
}


#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum StaticValue {
	Nil,
	Boolean(bool),
	Number(P8Num),
	String,
	Table,
	Function,
	Thread,
	UserData,
}

// todo: context for strings / tables / maybe function names
pub fn to_static_value(x: &Value) -> StaticValue {
	match x {
		Value::Nil          => StaticValue::Nil,
		Value::Boolean(b)   => StaticValue::Boolean(b.clone()),
		Value::Number(n)    => StaticValue::Number(*n),
		Value::String(_s)   => StaticValue::String,
		Value::Table(_t)    => StaticValue::Table,
		Value::Function(_f) => StaticValue::Function,
		Value::Thread(_)    => StaticValue::Thread,
		Value::UserData(_u) => StaticValue::UserData,
	}
}


#[cfg(test)]
mod test {
	use crate::pico8::{to_static_value, Pico8VM, RunResult, StaticValue};
	use alloc::vec::Vec;
	use p8rs_macros::p8;
	use p8rs_piccolo::{Closure, Executor, Value, Variadic};
	
	#[test]
	pub fn it_works() {
		let source = b"
			local a = 30
			return a + 9
		";
		
		let mut vm = Pico8VM::new().unwrap();
		
		let ex = vm.lua.try_enter(|ctx| {
			let env = ctx.globals();
			let closure = Closure::load_with_env(ctx, None, &source[..], env)?;
			let ex = Executor::start(ctx, closure.into(), ());
			
			Ok(ctx.stash(ex))
		}).unwrap();
		
		vm.lua.finish(&ex, &mut ()).unwrap();
		
		let res = vm.lua.try_enter(|ctx| {
			let exec = ctx.fetch(&ex);
			let vals = exec.take_result::<Variadic<Vec<Value>>>(ctx)??;
			
			let results = vals.iter().map(to_static_value).collect::<Vec<_>>();
			
			Ok(results)
		}).unwrap();
		
		assert_eq!(res.len(), 1, "expected one result");
		let first = res[0];
		if let StaticValue::Number(i) = first {
			assert_eq!(i, p8!(39), "first element should be 39");
		}
		
		info!("XDD {:?}", res);
	}

	#[test]
	pub fn test_cartridge_loading() {
		let cartridge_content = r#"pico-8 cartridge // http://www.pico-8.com
version 8

__lua__
printh("Hello from cartridge!")
return 42
"#;

		let mut vm = Pico8VM::new().unwrap();
		assert!(vm.load_cartridge(cartridge_content.as_bytes()).is_ok());

		// Run the cartridge and verify it executes
		let result = vm.run();
		assert!(matches!(result, Ok(RunResult::Stop))); // Should complete successfully
	}
}
