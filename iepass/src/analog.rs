use std::borrow::Borrow;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::gpio::ADCPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::sys::EspError;

use crate::calib::{Axes, BiRange};

pub struct Analog<'d, PinX, PinY, Adc>
where PinX: ADCPin,
      PinY: ADCPin,
      Adc: Borrow<AdcDriver<'d, PinX::Adc>> + Borrow<AdcDriver<'d, PinY::Adc>> {
	pub x: AdcChannelDriver<'d, PinX, Adc>,
	pub y: AdcChannelDriver<'d, PinY, Adc>,
	pub deadzone: u16,
	pub calib: Axes<BiRange<u16>>,
}

impl<'d, PinX, PinY, Adc> Analog<'d, PinX, PinY, Adc>
where PinX: ADCPin,
      PinY: ADCPin,
      Adc: Borrow<AdcDriver<'d, PinX::Adc>> + Borrow<AdcDriver<'d, PinY::Adc>> + Clone {
	pub fn new(
		adc: Adc,
		pin_x: impl Peripheral<P = PinX> + 'd,
		pin_y: impl Peripheral<P = PinY> + 'd,
		deadzone: u16,
		calib: Axes<BiRange<u16>>,
	) -> Result<Self, EspError> {
		Ok(Self {
			x: AdcChannelDriver::new(adc.clone(), pin_x, &Default::default())?,
			y: AdcChannelDriver::new(adc.clone(), pin_y, &Default::default())?,
			deadzone,
			calib,
		})
	}
	
	pub fn read_raw(&mut self) -> Result<(u16, u16), EspError> {
		let x = self.x.read()?;
		let y = self.y.read()?;
		
		Ok((x, y))
	}
	
	pub fn read(&mut self, scale: i16) -> Result<(i16, i16), EspError> {
		let (x, y) = self.read_raw()?;
		
		Ok(self.rescale(x, y, scale))
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
