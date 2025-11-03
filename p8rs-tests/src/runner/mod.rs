pub mod pico8;
pub mod p8rs;

use std::fmt::{Display, Formatter};
use crate::utils::str_splitn_array;

pub const TIMEOUT_MS: u64 = 5000;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Log {
	TEST(String, String),
	MEM(String, String),
	SCR(String, String),
	OTHER(String),
}

impl Log {
	pub fn name(&self) -> Option<&str> {
		match self {
			Log::TEST(name, _) => Some(name),
			Log::MEM(name, _) => Some(name),
			Log::SCR(name, _) => Some(name),
			Log::OTHER(_) => None,
		}
	}
}

impl From<&str> for Log {
	fn from(value: &str) -> Self {
		if let Some([kind, name, content]) = value.strip_prefix("INFO: ").and_then(|s| str_splitn_array(s, " | ")) {
			match kind {
				"TEST" => return Log::TEST(name.into(), content.into()),
				"MEM" => return Log::MEM(name.into(), content.into()),
				"SCR" => return Log::SCR(name.into(), content.into()),
				_ => {}
			}
		}
		
		Log::OTHER(value.into())
	}
}

impl Display for Log {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Log::TEST(name, content) => write!(f, "TEST | {} | {}", name, content),
			Log::MEM(name, content) => write!(f, "MEM | {} | {}", name, content),
			Log::SCR(name, content) => write!(f, "SCR | {} | {}", name, content),
			Log::OTHER(content) => write!(f, "{}", content),
		}
	}
}

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
