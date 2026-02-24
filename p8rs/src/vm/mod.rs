use alloc::alloc::Global;
use alloc::boxed::Box;
use core::alloc::Allocator;
use anyhow::anyhow;
use gc_arena::Mutation;
use p8rs_piccolo::table::InvalidTableKey;
use p8rs_piccolo::{Closure, Error, Executor, ExecutorMode, ExternError, Fuel, Lua, RuntimeError, StashedExecutor, StashedTable, Table, Value};
use p8rs_types::p8num::P8Num;

pub mod memory;
pub mod palette;
pub mod font;
pub mod cart;
pub mod runtime;
pub mod callbacks;
mod api;
mod traceback;

pub use runtime::Runtime;
pub use callbacks::Callbacks;
use api::load_all;
use cart::CartLoadError;
use traceback::write_traceback_entries;
use crate::vm::memory::Memory;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RunResult {
	Flip,
	Stop,
	OutOfFuel,
}

pub struct P8rs<A: Allocator = Global> {
	lua: Lua,
	fresh_globals: StashedTable,
	runtime: Box<Runtime, A>,
	executor: Option<StashedExecutor>,
}

impl P8rs<Global> {
	pub fn new() -> Result<P8rs, InvalidTableKey> {
		Self::new_in(Global)
	}
}

impl<A: Allocator + 'static> P8rs<A> {
	pub fn new_in(alloc: A) -> Result<P8rs<A>, InvalidTableKey> {
		let mut lua = Lua::empty();
		
		let fresh_globals = lua.enter(|ctx| {
			load_all(ctx);
			Ok(ctx.stash(shallow_copy_table(ctx.mutation(), ctx.globals())?))
		})?;
		
		Ok(Self {
			lua,
			runtime: Runtime::new(alloc),
			executor: None,
			fresh_globals
		})
	}
	
	pub fn load_cartridge(&mut self, cartridge: impl AsRef<[u8]>) -> Result<(), CartLoadError> {
		cart::load_cartridge(self, cartridge.as_ref())
	}
	
	pub fn load_lua(&mut self, source: &[u8]) -> Result<(), ExternError> {
		let ex = self.lua.try_enter(|ctx| {
			let env = shallow_copy_table(ctx.mutation(), ctx.fetch(&self.fresh_globals))?;
			let kernel = Closure::load_with_env(ctx, None, include_bytes!("kernel.lua"), env)?;
			let closure = Closure::load_with_env(ctx, None, source, env)?;
			let ex = Executor::start(ctx, kernel.into(), closure);
			
			Ok(ctx.stash(ex))
		})?;
		
		self.executor = Some(ex);
		
		Ok(())
	}
	
	pub fn set_callbacks(&mut self, callbacks: impl Callbacks + 'static) {
		self.runtime.set_callbacks(callbacks);
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
					
					self.runtime.start_frame();
					
					if !executor.step(ctx, &mut fuel, &mut *self.runtime).unwrap() {
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
										Error::Lua(e) => error!("[run_fuel] Uncaught lua error ({}): {}", e.0.type_name(), e.0.display()),
										Error::Runtime(e) => {
											error!("[run_fuel] Uncaught runtime error: {}", e);
											if let Some(traceback) = &e.traceback {
												let entries = traceback.entries();
												let mut str = alloc::string::String::new();
												write_traceback_entries(&mut str, (&entries[..entries.len()-1]).iter())?;
												error!("{}", str);
											}
										},
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
	
	pub fn runtime(&mut self) -> &mut Runtime {
		&mut self.runtime
	}
	
	pub fn memory(&mut self) -> &mut Memory {
		&mut self.runtime.memory
	}
}

fn shallow_copy_table<'gc>(mc: &Mutation<'gc>, src: Table<'gc>) -> Result<Table<'gc>, InvalidTableKey> {
	let ret = Table::new(mc);
	for (k, v) in src.iter() {
		ret.set_raw(mc, k, v)?;
	}
	Ok(ret)
}

#[cfg(test)]
mod test {
	use crate::vm::{P8rs, RunResult};
	use alloc::vec::Vec;
	use p8rs_macros::p8;
	use p8rs_piccolo::{Closure, Executor, Value, Variadic};
	
	#[test]
	pub fn it_works() {
		let source = b"
			local a = 30
			return a + 9
		";
		
		let mut vm = P8rs::new().unwrap();
		
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
			
			assert_eq!(vals.len(), 1, "expected one result");
			let first = vals[0];
			if let Value::Number(i) = first {
				assert_eq!(i, p8!(39), "first element should be 39");
			}
			
			Ok(())
		}).unwrap();
		
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

		let mut vm = P8rs::new().unwrap();
		assert!(vm.load_cartridge(cartridge_content.as_bytes()).is_ok());

		// Run the cartridge and verify it executes
		let result = vm.run();
		assert!(matches!(result, Ok(RunResult::Stop))); // Should complete successfully
	}
}
