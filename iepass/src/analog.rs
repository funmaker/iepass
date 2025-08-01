use std::borrow::Borrow;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::gpio::ADCPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::sys::EspError;

pub struct Analog<'d, PinX, PinY, Adc>
where PinX: ADCPin,
      PinY: ADCPin,
      Adc: Borrow<AdcDriver<'d, PinX::Adc>> + Borrow<AdcDriver<'d, PinY::Adc>> {
	pub x: AnalogAxis<'d, PinX, Adc>,
	pub y: AnalogAxis<'d, PinY, Adc>,
	pub deadzone: u16,
}

impl<'d, PinX, PinY, Adc> Analog<'d, PinX, PinY, Adc>
where PinX: ADCPin,
      PinY: ADCPin,
      Adc: Borrow<AdcDriver<'d, PinX::Adc>> + Borrow<AdcDriver<'d, PinY::Adc>> + Clone {
	pub fn new(
		adc: Adc,
		pin_x: impl Peripheral<P = PinX> + 'd,
		pin_y: impl Peripheral<P = PinY> + 'd,
	) -> Result<Self, EspError> {
		Ok(Self {
			x: AnalogAxis {
				driver: AdcChannelDriver::new(adc.clone(), pin_x, &Default::default())?,
				min: 0,
				mid: 232,
				max: 450,
			},
			y: AnalogAxis {
				driver: AdcChannelDriver::new(adc.clone(), pin_y, &Default::default())?,
				min: 0,
				mid: 232,
				max: 450,
			},
			deadzone: 4,
		})
	}
	
	pub fn read(&mut self, scale: i16) -> Result<(i16, i16), EspError> {
		let x = self.x.driver.read()?;
		let y = self.y.driver.read()?;
		
		if x.abs_diff(self.x.mid) < self.deadzone && y.abs_diff(self.y.mid) < self.deadzone { return Ok((0, 0)) }
		
		fn rescale(value: u16, min: u16, mid: u16, max: u16, scale: i16) -> i16 {
			if value < min { -scale }
			else if value < mid { ((mid - value) as i32 * -scale as i32 / (mid - min) as i32) as i16 }
			else if value < max { ((value - mid) as i32 * scale as i32 / (max - mid) as i32) as i16 }
			else { scale }
		}
		
		Ok((
			rescale(x, self.x.min, self.x.mid, self.x.max, scale),
			rescale(y, self.y.min, self.y.mid, self.y.max, scale),
		))
	}
}

pub struct AnalogAxis<'d, Pin, Adc>
where Pin: ADCPin,
      Adc: Borrow<AdcDriver<'d, Pin::Adc>> {
	pub driver: AdcChannelDriver<'d, Pin, Adc>,
	pub min: u16,
	pub mid: u16,
	pub max: u16,
}
