#![feature(mpmc_channel)]

use std::process::{Child, exit};
use std::sync::mpsc;
use std::thread;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;

pub mod perf_msg;
pub mod probe;
pub mod graphs;
mod elf;
mod utils;

use graphs::Graphs;

const RUNNER: &'static str = "probe-rs";

fn main() {
	let args: Vec<_> = std::env::args_os().skip(1).collect();
	let binary_path = args.last().unwrap();
	
	if let Err(err) = ctrlc::set_handler(ctrlc_handler) {
		eprintln!("Failed to set up Ctrl-C handler. Backtrace might be missing: {err}");
	}
	
	let (sender, receiver) = mpsc::channel();
	let probe = probe::spawn(&args, sender);
	
	thread::spawn(move || wait_for_exit(probe));
	
	let get_symbols = ||
		elf::get_symbols(binary_path)
			.inspect_err(|err| eprintln!("Failed to load symbols: {err}"))
			.unwrap_or_default();
	
	Graphs::wait_for_data(receiver, get_symbols);
}

static CTRLC_RECEIVED: AtomicBool = AtomicBool::new(false);

fn ctrlc_handler() {
	if CTRLC_RECEIVED.swap(true, Ordering::Relaxed) {
		eprintln!("Received second Ctrl+C, exiting.");
		exit(-1);
	} else {
		eprintln!("Received Ctrl+C, waiting for probe-rs to exit.");
	}
}

fn wait_for_exit(mut probe: Child) {
	loop {
		if let Some(status) = probe.try_wait().expect("Failed to wait for probe-rs process.") {
			exit(status.code().unwrap_or(0));
		} else {
			thread::yield_now();
		}
	}
}
