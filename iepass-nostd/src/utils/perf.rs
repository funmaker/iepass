use alloc::format;
use alloc::string::String;
use core::cell::RefCell;
use core::fmt::Write;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex};
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Instant;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use pin_project::pin_project;
use rtt_target::UpChannel;

const PERF_SIZE: usize = 100;
static PERF_BUF: Mutex<CriticalSectionRawMutex, RefCell<PerfInner>> = Mutex::new(RefCell::new(PerfInner::new()));

#[allow(dead_code)]
pub fn set_channel(output: UpChannel) {
	PERF_BUF.lock(|inner| {
		inner.borrow_mut().output = Some(output);
	})
}

pub fn dump_perf() {
	PERF_BUF.lock(|inner| {
		let PerfInner { ref entries, ref mut output } = *inner.borrow_mut();
		
		match output.as_mut() {
			Some(channel) => PerfInner::write_entries(entries, channel),
			None => {
				let mut output = String::new();
				PerfInner::write_entries(entries, &mut output);
				defmt::println!("[PERF ] {}", output.trim_end_matches("\n"));
			},
		};
	})
}

struct PerfInner {
	entries: ConstGenericRingBuffer<Entry, PERF_SIZE>,
	output: Option<UpChannel>,
}

impl PerfInner {
	const fn new() -> Self {
		Self {
			entries: ConstGenericRingBuffer::new(),
			output: None,
		}
	}
	
	pub fn write_entries<W: Write>(entries: &ConstGenericRingBuffer<Entry, PERF_SIZE>, mut output: W) {
		let time_epoch = entries.iter()
		                        .map(|entry| entry.start)
		                        .min()
		                        .unwrap_or(Instant::now());
		
		write!(output, "[").unwrap();
		for (pos, entry) in entries.iter().enumerate() {
			if pos != 0 { write!(output, ",").unwrap(); }
			write!(output, "[\"{}\",{},{}]", entry.name, entry.start.duration_since(time_epoch).as_micros(), entry.end.duration_since(time_epoch).as_micros()).unwrap()
		}
		write!(output, "]\n").unwrap();
	}
}

#[derive(Debug, Clone, Copy)]
struct Entry {
	name: &'static str,
	start: Instant,
	end: Instant,
}

#[pin_project]
pub struct PerfFuture<F> {
	name: &'static str,
	#[pin] inner: F,
}

impl<F: Future> PerfFuture<F> {
	pub fn new(name: &'static str, inner: F) -> Self {
		Self { name, inner }
	}
}

impl<F: Future> Future for PerfFuture<F> {
	type Output = F::Output;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let name = self.name;
		let start = Instant::now();
		
		let result = self.project().inner.poll(cx);
		
		let end = Instant::now();
		PERF_BUF.lock(|buf|
			buf.borrow_mut().entries.enqueue(Entry { name, start, end })
		);
		
		result
	}
}

pub trait PerfFutureExt: Future + Sized {
	fn perf_name(self, name: &'static str) -> PerfFuture<Self>;
}

impl<F: Future> PerfFutureExt for F {
	fn perf_name(self, name: &'static str) -> PerfFuture<Self> {
		PerfFuture::new(name, self)
	}
}
