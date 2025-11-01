use std::cell::RefCell;
use std::rc::Rc;
use iepass_core::pico8;
use p8rs_types::p8scii;

use crate::runner::{Log, RunResult};

struct RunnerCallback {
	buffer: Rc<RefCell<String>>,
}

impl pico8::Callbacks for RunnerCallback {
	fn printh(&mut self, text: &[u8], _filename: Option<&[u8]>, _overwrite: Option<bool>, _save_to_desktop: Option<bool>) {
		let mut buffer = self.buffer.borrow_mut();
		*buffer += "INFO: ";
		buffer.extend(p8scii::to_iter(text));
		*buffer += "\n";
	}
}

pub fn run(source: &[u8]) -> RunResult {
	let output = Rc::new(RefCell::new(String::new()));
	let mut vm = pico8::Pico8VM::new().expect("Failed to create P8rs VM");
	vm.set_callbacks(RunnerCallback { buffer: output.clone() });
	vm.load_cartridge(source).expect("Failed to load cartridge");
	
	let result = loop {
		match vm.run() {
			Ok(pico8::RunResult::Flip) => continue,
			Ok(pico8::RunResult::Stop) => break Ok(()),
			Err(err) => break Err(err),
			_ => unreachable!(),
		}
	};
	
	let output = &*output.borrow();
	let output = output.trim();
	
	println!("-- p8rs output --");
	println!("{output}");
	println!();
	if let Some(err) = result.as_ref().err() {
		println!("-- p8rs error --");
		println!("{err}");
		println!();
	}
	
	let logs = output.lines().map(Log::from).collect();
	let runtime_error = result.err().map(|err| err.to_string());
	
	RunResult::new(logs, runtime_error)
}
