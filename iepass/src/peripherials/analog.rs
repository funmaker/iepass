use esp_hal::analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, Attenuation};
use esp_hal::Blocking;
use esp_hal::gpio::AnalogPin;
use esp_hal::peripherals::ADC2;

use crate::calib::{Axes, BiRange};

pub struct Analog<'d, PinX, PinY>
where PinX: AdcChannel,
      PinY: AdcChannel {
	pub adc: Adc<'d, ADC2<'d>, Blocking>,
	pub x: AdcPin<PinX, ADC2<'d>>,
	pub y: AdcPin<PinY, ADC2<'d>>,
	pub deadzone: u16,
	pub calib: Axes<BiRange<u16>>,
}

impl<'d, PinX, PinY> Analog<'d, PinX, PinY>
where PinX: AnalogPin + AdcChannel + 'd,
      PinY: AnalogPin + AdcChannel + 'd {
	pub fn new(
		adc: ADC2<'d>,
		x: PinX,
		y: PinY,
		deadzone: u16,
		calib: Axes<BiRange<u16>>,
	) -> Self {
		let mut config = AdcConfig::new();
		let x = config.enable_pin(x, Attenuation::_0dB);
		let y = config.enable_pin(y, Attenuation::_0dB);
		let adc = Adc::new(adc, config);
		
		Self {
			adc,
			x,
			y,
			deadzone,
			calib,
		}
	}
	
	pub fn read_raw(&mut self) -> (u16, u16) {
		let x = self.adc.read_blocking(&mut self.x);
		let y = self.adc.read_blocking(&mut self.y);
		
		(x, y)
	}
	
	pub fn read(&mut self, scale: i16) -> (i16, i16) {
		let (x, y) = self.read_raw();
		
		self.rescale(x, y, scale)
	}
	
	pub fn rescale(&self, x: u16, y: u16, scale: i16) -> (i16, i16) {
		if x.abs_diff(self.calib.x.mid) < self.deadzone && y.abs_diff(self.calib.y.mid) < self.deadzone { return (0, 0) }
		
		(
			rescale_axis(x, self.calib.x.min, self.calib.x.mid, self.calib.x.max, scale),
			rescale_axis(y, self.calib.y.min, self.calib.y.mid, self.calib.y.max, scale),
		)
	}
}

fn rescale_axis(value: u16, min: u16, mid: u16, max: u16, scale: i16) -> i16 {
	if value <= min { -scale }
	else if value < mid { ((mid - value) as i32 * -scale as i32 / (mid - min) as i32) as i16 }
	else if value < max { ((value - mid) as i32 * scale as i32 / (max - mid) as i32) as i16 }
	else { scale }
}
