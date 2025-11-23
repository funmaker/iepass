use std::borrow::Cow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RunResult {
	pub output: String,
	pub runtime_error: Option<String>,
	pub timeout: bool,
}

impl RunResult {
	pub fn new(output: String, runtime_error: Option<String>, timeout: bool) -> Self {
		RunResult {
			output,
			runtime_error,
			timeout
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary<'a> {
	pub pico8: SummarySubject<'a>,
	pub p8rs: SummarySubject<'a>,
	pub orig_cart_path: Cow<'a, str>,
	pub tmp_cart_path: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySubject<'a> {
	pub log_name: Cow<'a, str>,
	pub runtime_error: Option<Cow<'a, str>>,
	pub timeout: bool,
}

impl<'a> SummarySubject<'a> {
	pub fn new(log_name: &'a str, run_result: &'a RunResult) -> Self {
		Self {
			log_name: Cow::Borrowed(log_name),
			runtime_error: run_result.runtime_error.as_deref().map(|err: &str| err.into()),
			timeout: run_result.timeout,
		}
	}
}
