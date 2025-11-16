use crate::log::Log;

pub mod pico8;
pub mod p8rs;

pub const TIMEOUT_MS: u64 = 5000;

#[derive(Debug, Clone)]
pub struct RunResult {
	pub logs: Vec<Log>,
	pub runtime_error: Option<String>,
	pub timeout: bool,
}

impl RunResult {
	pub fn new(logs: Vec<Log>, runtime_error: Option<String>, timeout: bool) -> Self {
		RunResult { logs, runtime_error, timeout }
	}
}
