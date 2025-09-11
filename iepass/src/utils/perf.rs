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
use esp_hal::system::Cpu;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use pin_project::pin_project;
use rtt_target::UpChannel;

use crate::utils::PSRAM_ALLOCATOR;

const PERF_SIZE: usize = 64;
static PERF_BUF: Mutex<CriticalSectionRawMutex, RefCell<PerfInner>> = Mutex::new(RefCell::new(PerfInner::new()));

#[allow(dead_code)]
pub fn set_channel(output: UpChannel) {
	#[cfg(feature = "perf")]
	PERF_BUF.lock(|inner| {
		inner.borrow_mut().output = Some(output);
	})
}

pub fn dump_perf() -> Result<(), core::fmt::Error> {
	PERF_BUF.lock(|inner| {
		let PerfInner { ref entries, ref mut output } = *inner.borrow_mut();
		
		match output.as_mut() {
			Some(mut channel) => {
				PerfInner::write_entries(entries, &mut channel)?;
				write!(channel, "\n")?;
			},
			None => {
				let mut output = String::with_capacity(PERF_SIZE * 30);
				PerfInner::write_entries(entries, &mut output)?;
				defmt::println!("[PERF ] {}", output);
			},
		};
		
		Ok(())
	})?;
	
	Ok(())
}

pub fn sync_perf<R>(name: &'static str, func: impl FnOnce() -> R) -> R {
	let start = Instant::now();
	
	let ret = func();
	
	let end = Instant::now();
	PERF_BUF.lock(|buf| buf.borrow_mut().entries.enqueue(Entry { name, start, end, cpu: Cpu::current() }));
	
	ret
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
	
	pub fn write_entries<W: Write>(entries: &ConstGenericRingBuffer<Entry, PERF_SIZE>, mut output: W) -> Result<(), core::fmt::Error> {
		let time_epoch = entries.iter()
		                        .map(|entry| entry.start)
		                        .min()
		                        .unwrap_or(Instant::now());
		
		let sram = esp_alloc::HEAP.stats();
		let psram = PSRAM_ALLOCATOR.stats();
		
		defmt::info!("{}", sram);
		defmt::info!("{}", psram);
		
		let sram_used  = esp_alloc::HEAP.used();
		let sram_free  = esp_alloc::HEAP.free();
		let psram_used = PSRAM_ALLOCATOR.used();
		let psram_free = PSRAM_ALLOCATOR.free();
		
		write!(
			output,
			"{{\"sram\":[{},{}],\"psram\":[{},{}],\"trace\":[",
			sram_used,
			sram_used + sram_free,
			psram_used,
			psram_used + psram_free,
		)?;
		for (pos, entry) in entries.iter().enumerate() {
			if pos != 0 { write!(output, ",")?; }
			write!(
				output,
				"[\"{}\",{},{},{}]",
				entry.name,
				entry.start.duration_since(time_epoch).as_micros(),
				entry.end.duration_since(time_epoch).as_micros(),
				entry.cpu as usize,
			)?;
		}
		write!(output, "]}}")?;
		
		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
struct Entry {
	name: &'static str,
	start: Instant,
	end: Instant,
	cpu: Cpu,
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
		sync_perf(self.name, || self.project().inner.poll(cx))
	}
}

pub trait PerfFutureExt: Future + Sized {
	fn perf_trace(self, name: &'static str) -> PerfFuture<Self>;
}

impl<F: Future> PerfFutureExt for F {
	fn perf_trace(self, name: &'static str) -> PerfFuture<Self> {
		PerfFuture::new(name, self)
	}
}
