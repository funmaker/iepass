use std::io::Read;
use std::ops::Not;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::runner::{Log, RunResult, TIMEOUT_MS};

pub fn run(path: impl AsRef<Path>, log_file: impl AsRef<Path>) -> RunResult {
	let mut child =
		Command::new("pico8")
			.arg("-run")
			.arg(path.as_ref())
			.arg("-x")
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.expect("failed to execute pico8");
	
	let mut stdout = child.stdout.take().unwrap();
	let stdout = thread::spawn(move || {
		let mut output = String::new();
		stdout.read_to_string(&mut output).expect("failed to read pico8 stdout stream");
		return output;
	});
	
	let mut stderr = child.stderr.take().unwrap();
	let stderr = thread::spawn(move || {
		let mut output = String::new();
		stderr.read_to_string(&mut output).expect("failed to read pico8 stdout stream");
		return output;
	});
	
	let mut timeout = false;
	let start = Instant::now();
	loop {
		match child.try_wait().expect("failed to wait on pico8") {
			Some(_) => break,
			None => {},
		}
		
		thread::sleep(Duration::from_millis(10));
		if start.elapsed() > Duration::from_millis(TIMEOUT_MS) {
			timeout = true;
			child.kill().expect("failed to kill pico8");
			child.wait().expect("failed to wait after kill on pico8");
			break;
		}
	}
	
	let stdout = stdout.join().expect("Can't read pico8 stdout stream");
	let stderr = stderr.join().expect("Can't read pico8 stderr stream");
	
	println!("-- pico8 stdout --");
	println!("{stdout}");
	
	if timeout {
		println!("-- pico8 timed out --");
	}
	
	std::fs::write(&log_file, &stderr).expect("Can't write to pico8 output log");
	
	let stdout = if stdout.starts_with("RUNNING: ") {
		let eol = stdout.find('\n').unwrap_or(stdout.len());
		stdout[eol ..].trim()
	} else {
		stdout.trim()
	};
	
	let logs = stderr.lines().map(Log::from).collect();
	let runtime_error = stdout.is_empty().not().then_some(stdout.to_string());
	
	RunResult::new(logs, runtime_error, timeout)
}
