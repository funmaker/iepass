use std::thread;
use std::time::{Duration, Instant};
use std::panic;
use std::sync::{Arc, Mutex};
use p8rs::vm;
use p8rs::piccolo::ExternError;
use p8rs_types::p8scii;
use crate::runner::TIMEOUT_MS;
use crate::summary::RunResult;

#[derive(Debug)]
struct RunnerCallback {
	buffer: Arc<Mutex<String>>,
}

impl vm::Callbacks for RunnerCallback {
	fn printh(&mut self, text: &[u8], _filename: Option<&[u8]>, _overwrite: Option<bool>, _save_to_desktop: Option<bool>) {
		let mut buffer = self.buffer.lock().unwrap();
		if cfg!(windows) {
			*buffer += "INFO: ";
		}
		buffer.extend(p8scii::to_iter(text));
		*buffer += "\n";
	}
}

pub fn run(source: &[u8]) -> RunResult {
	let output = Arc::new(Mutex::new(String::new()));
	
	let start = Instant::now();
	let callback = RunnerCallback { buffer: output.clone() };
	let result = panic::catch_unwind(|| {
		let mut vm = vm::P8rs::new().expect("Failed to create P8rs VM");
		vm.set_callbacks(callback);
		
		if let Err(err) = vm.load_cartridge(source) {
			println!("-- p8rs load error --");
			println!("{err}");
			println!();
			
			return Err(err.to_string());
		}
		
		loop {
			match vm.run_fuel(1024 * 1024) {
				Ok(vm::RunResult::Flip) |
				Ok(vm::RunResult::OutOfFuel) => {}
				Ok(vm::RunResult::Stop) => break Ok(false),
				Err(ExternError::Lua(err)) => {
					println!("-- p8rs error --");
					println!("{err}");
					return Err(err.to_string());
				}
				Err(ExternError::Runtime(err)) => {
					println!("-- p8rs error --");
					println!("{err}");
					if let Some(traceback) = err.traceback.as_ref() {
						println!("traceback:\n{traceback}");
					}
					return Err(err.to_string());
				}
			}
			
			thread::sleep(Duration::from_millis(10));
			if start.elapsed() > Duration::from_millis(TIMEOUT_MS) {
				return Ok(true);
			}
		}
	});
	
	let output = output.lock().unwrap().clone();
	
	let (timeout, runtime_error) = match result {
		Ok(Ok(timeout)) => (timeout, None),
		Ok(Err(err)) => (false, Some(err)),
		Err(panic) => {
			if let Some(str) = panic.downcast_ref::<&str>() {
				println!("-- p8rs panic --");
				println!("{str:?}");
				(false, Some(str.to_string()))
			} else if let Some(str) = panic.downcast_ref::<String>() {
				println!("-- p8rs panic --");
				println!("{str:?}");
				(false, Some(str.to_string()))
			} else {
				println!("-- p8rs panic --");
				println!("Can't parse panic");
				(false, Some("Can't parse panic".to_string()))
			}
		}
	};
	
	if timeout {
		println!("-- p8rs timed out --");
	}
	
	RunResult::new(output, runtime_error, timeout)
}
