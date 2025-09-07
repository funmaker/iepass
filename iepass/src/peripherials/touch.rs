use core::array;
use anyhow::{anyhow, Result};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_hal_async::spi::{Operation, SpiDevice as _};
use bitflags::bitflags;
use esp_hal::Async;
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin};
use esp_hal::spi::master::SpiDmaBus;

use crate::peripherials::SpiBus;
use crate::calib::{Axes, Range};

const TOUCH_THRESHOLD: u16 = 10;

pub struct Touch<'d> {
	device: SpiDevice<'d, CriticalSectionRawMutex, SpiDmaBus<'d, Async>, Output<'d>>,
	_iqr: Input<'d>,
	calib: Axes<Range<u16>>,
}

impl<'d> Touch<'d> {
	pub fn new(spi_bus: &'d SpiBus<'d>,
	           cs: impl OutputPin + 'd,
	           iqr: impl InputPin + 'd,
	           calib: Axes<Range<u16>>)
	           -> Self {
		Self {
			device: SpiDevice::new(&*spi_bus, Output::new(cs, Level::High, OutputConfig::default())),
			_iqr: Input::new(iqr, InputConfig::default()),
			calib,
		}
	}
	
	#[allow(dead_code)]
	pub fn apply_calib(&mut self, calib: Axes<Range<u16>>) {
		self.calib = calib;
	}
	
	pub async fn read_raw(&mut self) -> Result<Option<(u16, u16)>> {
		let [x, y, z1] = self.commands([
			Command::ADDR_X  | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL,
			Command::ADDR_Y  | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL,
			Command::ADDR_Z1 | Command::BIT_12 | Command::REF_DIFF | Command::POW_ALL,
		]).await?;
		
		if z1 < TOUCH_THRESHOLD {
			return Ok(None);
		}
		
		Ok(Some((x, y)))
	}
	
	pub async fn read(&mut self, scale_x: u16, scale_y: u16) -> Result<Option<(u16, u16)>> {
		self.read_raw()
			.await
			.map(|result| result.map(|(x, y)| (
				scale_axis(x, self.calib.x.min, self.calib.x.max, scale_x),
				scale_axis(y, self.calib.y.min, self.calib.y.max, scale_y),
			)))
	}
	
	async fn commands<const N: usize>(&mut self, commands: [Command; N]) -> Result<[u16; N]>
	where [(); N * 3]: {
		let mut buffer = [0u8; N * 3];
		let mut offset = 0;
		
		for command in commands {
			buffer[offset] = command.bits;
			if command.contains(Command::BIT_8) {
				offset += 2;
			} else {
				offset += 3;
			}
		}
		
		self.device.transaction(&mut [Operation::TransferInPlace(&mut buffer[0..offset])])
		    .await
		    .map_err(|err| anyhow!("{err:?}"))?;
		
		offset = 0;
		
		Ok(array::from_fn(|pos| {
			if commands[pos].contains(Command::BIT_8) {
				offset += 2;
				buffer[offset - 1] as u16
			} else {
				offset += 3;
				u16::from_be_bytes([buffer[offset - 2], buffer[offset - 1]]) >> 3
			}
		}))
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
		const ADDR_Z1    = 0b1011_0000;
		const ADDR_Z2    = 0b1100_0000;
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
