use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command};
use std::sync::mpsc::Sender;
use std::thread;
use colored::Colorize;

use crate::perf_msg::RawPerfMessage;
use crate::RUNNER;

pub fn spawn(args: &[OsString], sender: Sender<RawPerfMessage>) -> Child {
	use std::process::Stdio;
	
	println!("     {} `{} {}`", "Running".green().bold(), RUNNER, args.join(OsStr::new(" ")).to_string_lossy());
	let mut probe = Command::new("probe-rs").args(args).env("CLICOLOR_FORCE", "true").stdout(Stdio::piped()).spawn().unwrap();
	let probe_out = probe.stdout.take().unwrap();
	
	let perf_prefix = format!("{}{}{}", "[".bold(), "PERF ".cyan(), "]".bold());
	thread::spawn(move || {
		for line in BufReader::new(probe_out).lines() {
			let line = line.unwrap();
			
			if let Some(line) = line.strip_prefix("[perf ] ") {
				match serde_json::from_str::<RawPerfMessage>(line) {
					Ok(entries) => {
						println!("{perf_prefix} Parsed {} entries", entries.trace.len());
						sender.send(entries).unwrap()
					},
					Err(err) => println!("{perf_prefix} Can't parse PERF message:\n{}", err),
				}
			} else {
				println!("{}", line.trim_prefix("[defmt] "));
			}
		}
	});
	
	probe
}
