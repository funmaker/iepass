use std::time::{Duration, Instant};
use esp_idf_svc::hal::gpio::{InputMode, InputPin, OutputPin, Pin, PinDriver, Pull};
use esp_idf_svc::sys::EspError;

pub struct Debounce<'d, T, Mode>
	where T: Pin,
	      Mode: InputMode {
	pub inner: PinDriver<'d, T, Mode>,
	debounce_time: Duration,
	last_change: Instant,
	last_is_high: bool,
}

#[allow(dead_code)]
impl<'d, T, Mode> Debounce<'d, T, Mode>
	where T: Pin,
	      Mode: InputMode {
	pub fn new(inner: PinDriver<'d, T, Mode>) -> Self {
		Self {
			last_change: Instant::now(),
			last_is_high: inner.is_high(),
			debounce_time: Duration::from_millis(10),
			inner,
		}
	}
	
	pub fn with_time(mut self, time: Duration) -> Self {
		self.debounce_time = time;
		self
	}
	
	pub fn with_pull(mut self, pull: Pull) -> Result<Self, EspError>
	where T: InputPin + OutputPin {
		self.inner.set_pull(pull)?;
		Ok(self)
	}
	
	pub fn raising_edge(&mut self) -> bool {
		let changed = self.update();
		
		changed && self.last_is_high
	}
	
	pub fn falling_edge(&mut self) -> bool {
		let changed = self.update();
		
		changed && !self.last_is_high
	}
	
	pub fn is_low(&mut self) -> bool {
		self.update();
		
		!self.last_is_high
	}
	
	pub fn is_high(&mut self) -> bool {
		self.update();
		
		self.last_is_high
	}
	
	fn update(&mut self) -> bool {
		if self.last_change.elapsed() > self.debounce_time {
			let current_value = self.inner.is_high();
			
			if current_value != self.last_is_high {
				self.last_is_high = current_value;
				self.last_change = Instant::now();
				
				return true;
			}
		}
		
		false
	}
}
