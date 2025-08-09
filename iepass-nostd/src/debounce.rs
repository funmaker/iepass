use embassy_time::{Duration, Instant};
use esp_hal::gpio::{Input, Level};

pub struct Debounce<'d> {
	pub inner: Input<'d>,
	debounce_time: Duration,
	last_change: Instant,
	last_level: Level,
}

#[allow(dead_code)]
impl<'d> Debounce<'d> {
	pub fn new(inner: Input<'d>) -> Self {
		Self {
			last_change: Instant::now(),
			last_level: inner.level(),
			debounce_time: Duration::from_millis(10),
			inner,
		}
	}
	
	pub fn with_time(mut self, time: Duration) -> Self {
		self.debounce_time = time;
		self
	}
	
	pub fn raising_edge(&mut self) -> bool {
		let changed = self.update();
		
		changed && self.last_level == Level::High
	}
	
	pub fn falling_edge(&mut self) -> bool {
		let changed = self.update();
		
		changed && self.last_level == Level::Low
	}
	
	pub fn is_low(&mut self) -> bool {
		self.update();
		
		!self.last_level == Level::High
	}
	
	pub fn is_high(&mut self) -> bool {
		self.update();
		
		self.last_level == Level::Low
	}
	
	fn update(&mut self) -> bool {
		if self.last_change.elapsed() > self.debounce_time {
			let current_level = self.inner.level();
			
			if current_level != self.last_level {
				self.last_level = current_level;
				self.last_change = Instant::now();
				
				return true;
			}
		}
		
		false
	}
}
