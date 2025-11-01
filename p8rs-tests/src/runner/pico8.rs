use std::ops::Not;
use std::path::Path;
use std::process::Command;

use crate::runner::{Log, RunResult};

pub fn run(path: impl AsRef<Path>) -> RunResult {
	let output =
		Command::new("pico8")
			.arg("-run")
			.arg(path.as_ref())
			.arg("-x")
			.output()
			.expect("failed to execute pico8");
	
	let stdout = str::from_utf8(&output.stdout).expect("Invalid utf-8 in pico8 stdout stream").trim();
	let stderr = str::from_utf8(&output.stderr).expect("Invalid utf-8 in pico8 stderr stream").trim();
	
	println!("-- pico8 stderr --");
	println!("{stderr}");
	println!();
	println!("-- pico8 stdout --");
	println!("{stdout}");
	println!();
	
	let stdout = if stdout.starts_with("RUNNING: ") {
		let eol = stdout.find('\n').unwrap_or(stdout.len());
		stdout[eol ..].trim()
	} else {
		stdout.trim()
	};
	
	let logs = stderr.lines().map(Log::from).collect();
	let runtime_error = stdout.is_empty().not().then_some(stdout.to_string());
	
	RunResult::new(logs, runtime_error)
}
