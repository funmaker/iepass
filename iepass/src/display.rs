use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, Output, OutputPin, PinDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::MegaHertz;
use esp_idf_svc::hal::spi::{Dma, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::sys::EspError;
use st7735_lcd::{Orientation, ST7735};

use crate::calib::Axes;
use crate::utils::{SCR_HEIGHT, SCR_WIDTH};

pub const UNKNOWN_ERROR: EspError = EspError::from(-1).unwrap();

pub struct Display<'d, DC, RST>
where DC: OutputPin,
      RST: OutputPin {
	driver: ST7735<
		SpiDeviceDriver<'d, SpiDriver<'d>>,
		PinDriver<'d, DC, Output>,
		PinDriver<'d, RST, Output>,
	>,
}

#[allow(dead_code)]
impl<'d, DC, RST> Display<'d, DC, RST>
where DC: OutputPin, RST: OutputPin {
	pub fn new(
		spi: impl Peripheral<P = impl SpiAnyPins> + 'd,
		sclk: impl Peripheral<P = impl OutputPin> + 'd,
		sdo: impl Peripheral<P = impl OutputPin> + 'd,
		a0: DC,
		rst: RST,
		calib: Axes<u16>
	) -> Result<Self, EspError> {
		let spi = SpiDeviceDriver::new_single(
			spi,
			sclk,
			sdo,
			AnyIOPin::none(),
			AnyIOPin::none(),
			&DriverConfig {
				dma: Dma::Auto(SCR_WIDTH as usize * SCR_HEIGHT as usize * 2),
				..Default::default()
			},
			&SpiConfig {
				baudrate: MegaHertz(30).into(),
				..Default::default()
			}
		)?;
		
		let rst = PinDriver::output(rst)?;
		let a0 = PinDriver::output(a0)?;
		
		let mut driver = ST7735::new(
			spi,
			a0,
			rst,
			true,
			false,
			SCR_WIDTH as u32,
			SCR_HEIGHT as u32,
		);
		
		driver.set_offset(calib.x, calib.y);
		driver.init(&mut FreeRtos).map_err(|_| UNKNOWN_ERROR)?;
		driver.set_orientation(&Orientation::Landscape).map_err(|_| UNKNOWN_ERROR)?;
		driver.set_offset(calib.x, calib.y);
		driver.set_address_window(0, 0, SCR_WIDTH - 1, SCR_HEIGHT - 1).map_err(|_| UNKNOWN_ERROR)?;
		
		Ok(Self { driver })
	}
	
	pub fn set_calib(&mut self, calib: Axes<u16>) -> Result<(), EspError> {
		self.driver.set_offset(calib.x, calib.y);
		self.driver.set_address_window(0, 0, SCR_WIDTH - 1, SCR_HEIGHT - 1).map_err(|_| UNKNOWN_ERROR)?;
		
		Ok(())
	}
	
	pub fn update(&mut self, pixels: &[u16]) -> Result<(), EspError> {
		assert_eq!(pixels.len(), SCR_WIDTH as usize * SCR_HEIGHT as usize);
		
		self.driver.write_pixels_buffered(pixels.iter().copied()).map_err(|_| UNKNOWN_ERROR)?;
		
		Ok(())
	}
}
