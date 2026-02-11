use std::io::Read;
use std::ops::Not;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::runner::TIMEOUT_MS;
use crate::summary::RunResult;

const ERR_PREFIX: [&str; 3] = [
	"runtime error",
	"syntax error",
	"could not load",
];

pub fn run(path: impl AsRef<Path>) -> RunResult {
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
		stderr.read_to_string(&mut output).expect("failed to read pico8 stderr stream");
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
	print_trunc(&stdout, 10);
	println!();
	
	println!("-- pico8 stderr --");
	print_trunc(&stderr, 10);
	println!();
	
	if timeout {
		println!("-- pico8 timed out --");
	}
	
	let stdout = if stdout.starts_with("RUNNING: ") {
		let eol = stdout.find('\n').unwrap_or(stdout.len()) + 1;
		&stdout[eol ..]
	} else {
		&stdout[..]
	};
	
	if stderr.is_empty() { // Linux (everything goes to stdout)
		if let Some(err_line) = stdout.lines().find(|line| ERR_PREFIX.iter().any(|prefix| line.starts_with(prefix))) {
			let err_start = stdout.substr_range(err_line).unwrap().start;
			RunResult::new(&stdout[..err_start], Some(&stdout[err_start..]), timeout)
		} else {
			RunResult::new(stdout, None::<String>, timeout)
		}
	} else { // Windows (printh -> stderr, errors -> stdout)
		let runtime_error = stdout.is_empty().not().then_some(stdout.to_string());
		RunResult::new(stderr, runtime_error, timeout)
	}
}

fn print_trunc(text: &str, max_lines: usize) {
	let mut lines = text.lines();
	for line in (&mut lines).take(max_lines.saturating_sub(1)) {
		println!("{line}");
	}
	let remaining = lines.clone().count();
	if remaining > 1 {
		println!("... ({remaining} more)");
	} else if remaining == 1 {
		println!("{}", lines.next().unwrap());
	}
}
