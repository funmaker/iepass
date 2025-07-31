use std::borrow::Borrow;
use std::ops::Range;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::gpio::ADCPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::sys::EspError;

pub struct Analog<'d, Pin, Adc>
where Pin: ADCPin,
      Adc: Borrow<AdcDriver<'d, Pin::Adc>> {
	driver: AdcChannelDriver<'d, Pin, Adc>,
	range: Range<u16>,
	deadzone: u16,
}

#[allow(dead_code)]
impl<'d, Pin, Adc> Analog<'d, Pin, Adc>
where Pin: ADCPin,
      Adc: Borrow<AdcDriver<'d, Pin::Adc>> {
	pub fn new(
		adc: Adc,
		pin: impl Peripheral<P = Pin> + 'd,
	) -> Result<Self, EspError> {
		Ok(Self {
			driver: AdcChannelDriver::new(adc, pin, &Default::default())?,
			range: 0..460,
			deadzone: 4,
		})
	}
	
	pub fn with_range(mut self, range: Range<u16>, deadzone: u16) -> Self {
		self.range = range;
		self.deadzone = deadzone;
		self
	}
	
	pub fn read(&mut self, scale: i16) -> Result<i16, EspError> {
		let mv = self.driver.read()?;
		let low_min = self.deadzone;
		let low_max = (self.range.end - self.range.start) / 2 + self.range.start - self.deadzone / 2;
		let high_min = (self.range.end - self.range.start) / 2 + self.range.start + self.deadzone / 2;
		let high_max = self.range.end - self.deadzone;
		let scaled =
			if mv < low_min { -scale }
			else if mv < low_max { ((low_max - mv) as i32 * -scale as i32 / (low_max - low_min) as i32) as i16 }
			else if mv < high_min { 0 }
			else if mv < high_max { ((mv - high_min) as i32 * scale as i32 / (high_max - high_min) as i32) as i16 }
			else { scale };
		Ok(scaled)
	}
}

