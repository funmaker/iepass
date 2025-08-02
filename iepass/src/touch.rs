use std::borrow::Borrow;
use std::slice;
use bitflags::bitflags;
use esp_idf_svc::hal::gpio::{Input, InputPin, OutputPin, PinDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::MegaHertz;
use esp_idf_svc::hal::spi::{Operation, SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_svc::sys::EspError;

use crate::calib::{Axes, Range};

pub struct Touch<'d, Spi, IQR>
where Spi: Borrow<SpiDriver<'d>> + 'd,
      IQR: InputPin {
	pub spi: SpiDeviceDriver<'d, Spi>,
	pub iqr: PinDriver<'d, IQR, Input>,
	pub calib: Axes<Range<u16>>,
}

impl<'d, Spi, IQR> Touch<'d, Spi, IQR>
where Spi: Borrow<SpiDriver<'d>> + 'd,
      IQR: InputPin + OutputPin {
	pub fn new(spi: Spi,
	           cs: Option<impl Peripheral<P = impl OutputPin> + 'd>,
	           iqr: IQR,
	           calib: Axes<Range<u16>>)
		       -> Result<Self, EspError> {
		Ok(Self {
			spi: SpiDeviceDriver::new(
				spi,
				cs,
				&SpiConfig {
					baudrate: MegaHertz(10).into(),
					..Default::default()
				},
			)?,
			iqr: PinDriver::input(iqr)?,
			calib,
		})
	}
	
	pub fn read_raw(&mut self) -> Result<Option<(u16, u16)>, EspError> {
		self.command(Command::ADDR_AUX | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL)?;
		if self.iqr.is_high() {
			return Ok(None);
		}
		
		let x = self.command(Command::ADDR_X | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL)?;
		let y = self.command(Command::ADDR_Y | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL)?;
		
		Ok(Some((x, y)))
	}
	
	pub fn read(&mut self, scale_x: u16, scale_y: u16) -> Result<Option<(u16, u16)>, EspError> {
		if let Some((x, y)) = self.read_raw()? {
			Ok(Some((
				scale_axis(x, self.calib.x.min, self.calib.x.max, scale_x),
				scale_axis(y, self.calib.y.min, self.calib.y.max, scale_y),
			)))
		} else {
			Ok(None)
		}
	}
	
	fn command(&mut self, command: Command) -> Result<u16, EspError> {
		if command.contains(Command::BIT_8) {
			let mut res = 0;
			self.spi.transaction(&mut [
				Operation::Write(&[command.bits]),
				Operation::Read(slice::from_mut(&mut res)),
			])?;
			Ok(res as u16)
		} else {
			let mut res = [0, 0];
			self.spi.transaction(&mut [
				Operation::Write(&[command.bits]),
				Operation::Read(&mut res),
			])?;
			Ok(u16::from_be_bytes(res) >> 3)
		}
	}
}

fn scale_axis(value: u16, min: u16, max: u16, scale: u16) -> u16 {
	if value <= min { 0 }
	else if value >= max { scale }
	else {
		((value - min) as u32 * scale as u32 / (max - min) as u32) as u16
	}
}

bitflags! {
    struct Command: u8 {
        const ADDR_TEMP  = 0b1000_0000;
        const ADDR_X     = 0b1001_0000;
        const ADDR_BAT   = 0b1010_0000;
        const ADDR_Z1_X  = 0b1011_0000;
        const ADDR_Z2_Y  = 0b1100_0000;
        const ADDR_Y     = 0b1101_0000;
        const ADDR_AUX   = 0b1110_0000;
        const ADDR_TEMP2 = 0b1111_0000;
		
        const BIT_12 = 0b1000_0000;
        const BIT_8  = 0b1000_1000;
		
        const REF_DIFF   = 0b1000_0000;
        const REF_SINGLE = 0b1000_0100;
		
		const POW_DOWN = 0b1000_0000;
		const POW_ADC  = 0b1000_0001;
		const POW_REF  = 0b1000_0010;
		const POW_ALL  = 0b1000_0011;
    }
}
