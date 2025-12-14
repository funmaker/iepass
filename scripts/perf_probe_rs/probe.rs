use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command};
use std::sync::mpsc::Sender;
use std::thread;
use colored::Colorize;

use crate::perf_msg::RawPerfMessage;
use crate::RUNNER;

#[cfg(target_os = "linux")]
pub fn spawn(mut args: &[OsString], sender: Sender<RawPerfMessage>) -> Child {
	use ipipe::Pipe;
	
	let pipe = Pipe::with_name("iepass_perf").unwrap();
	let mut args = args.to_vec();
	args.push("--target-output-file".into());
	args.push(pipe.path().as_os_str().into());
	
	println!("     {} `{} {}`", "Running".green().bold(), RUNNER, args.join(OsStr::new(" ")).to_string_lossy());
	let probe = Command::new("probe-rs").args(args).spawn().unwrap();
	
	thread::spawn(move || {
		for line in BufReader::new(pipe).lines() {
			let line = line.unwrap();
			if let Some(line) = line.strip_prefix("[PERF ] ") {
				match serde_json::from_str(line) {
					Ok(entries) => sender.send(entries).unwrap(),
					Err(err) => eprintln!("Can't parse PERF message:\n{}", err),
				}
			}
		}
	});
	
	probe
}

#[cfg(not(target_os = "linux"))]
pub fn spawn(args: &[OsString], sender: Sender<RawPerfMessage>) -> Child {
	use std::process::Stdio;
	
	println!("     {} `{} {}`", "Running".green().bold(), RUNNER, args.join(OsStr::new(" ")).to_string_lossy());
	let mut probe = Command::new("probe-rs").args(args).env("CLICOLOR_FORCE", "true").stdout(Stdio::piped()).spawn().unwrap();
	let probe_out = probe.stdout.take().unwrap();
	
	thread::spawn(move || {
		for line in BufReader::new(probe_out).lines() {
			let line = line.unwrap();
			println!("{}", line);
			
			if let Some(line) = line.strip_prefix("[PERF ] ") {
				match serde_json::from_str(line) {
					Ok(entries) => sender.send(entries).unwrap(),
					Err(err) => eprintln!("Can't parse PERF message:\n{}", err),
				}
			}
		}
	});
	
	probe
}
