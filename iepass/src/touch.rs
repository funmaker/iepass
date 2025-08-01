use std::borrow::Borrow;
use std::slice;
use bitflags::bitflags;
use esp_idf_svc::hal::gpio::{Input, InputMode, InputPin, OutputPin, PinDriver, Pull};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::MegaHertz;
use esp_idf_svc::hal::spi::{Operation, SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_svc::sys::EspError;

pub struct Touch<'d, Spi, IQR>
where Spi: Borrow<SpiDriver<'d>> + 'd,
      IQR: InputPin {
	spi: SpiDeviceDriver<'d, Spi>,
	iqr: PinDriver<'d, IQR, Input>,
}

impl<'d, Spi, IQR> Touch<'d, Spi, IQR>
where Spi: Borrow<SpiDriver<'d>> + 'd,
      IQR: InputPin + OutputPin {
	pub fn new(spi: Spi,
	           cs: Option<impl Peripheral<P = impl OutputPin> + 'd>,
	           iqr: IQR)
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
		})
	}
	
	pub fn read(&mut self) -> Result<Option<(u16, u16)>, EspError> {
		// if self.iqr.is_high() {
		// 	return Ok(None);
		// }
		
		let x = self.command(Command::ADDR_X | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL)?;
		let y = self.command(Command::ADDR_Y | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL)?;
		
		log::info!("{x} {y} {}", self.iqr.is_high());
		
		Ok(Some((x, y)))
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
