//! Based on [st7735-lcd](https://github.com/sajattack/st7735-lcd-rs)

use embedded_hal_async::delay::DelayNs;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};
use esp_hal::spi::master::{Spi, AnySpi, Instance, Config, SpiDma};
use esp_hal::dma::{DmaChannelFor, DmaTxBuf};
use esp_hal::spi::{Mode};
use esp_hal::time::Rate;
use esp_hal::{dma_tx_buffer, Async};
use anyhow::{anyhow, bail, Result};

use crate::calib::Axes;
use crate::utils::framebuffer::Framebuffer;
use crate::utils::perf::sync_perf;

pub const WIDTH: u16 = 160;
pub const HEIGHT: u16 = 128;

pub struct Display<'d, D> {
	spi: Option<SpiDma<'d, Async>>,
	a0: Output<'d>,
	rst: Output<'d>,
	cmd_buf: Option<DmaTxBuf>,
	delay: D,
}

impl<'d, D: DelayNs> Display<'d, D> {
	pub async fn new(spi: impl Instance + 'd,
	                 dma: impl DmaChannelFor<AnySpi<'d>>,
	                 sclk: impl OutputPin + 'd,
	                 sdo: impl OutputPin + 'd,
	                 a0: impl OutputPin + 'd,
	                 rst: impl OutputPin + 'd,
	                 calib: Axes<u16>,
	                 delay: D)
	                 -> Result<Self> {
		let spi = Spi::new(spi, Config::default().with_frequency(Rate::from_mhz(40)).with_mode(Mode::_0))?
			.with_sck(sclk)
			.with_mosi(sdo)
			.with_dma(dma)
			.into_async();
		
		let a0 = Output::new(a0, Level::Low, OutputConfig::default());
		let rst = Output::new(rst, Level::High, OutputConfig::default());
		
		let cmd_buf = dma_tx_buffer!(64).map_err(|err| anyhow!("{:?}", err))?;
		
		let mut display = Display {
			spi: Some(spi),
			a0,
			rst,
			cmd_buf: Some(cmd_buf),
			delay,
		};
		
		display.init(calib).await?;
		
		Ok(display)
	}
	
	pub fn apply_calib(&mut self, calib: Axes<u16>) -> Result<()> {
		// Frame set up
		self.write_command(Instruction::CASET, &[calib.x.to_be_bytes(), (calib.x + WIDTH  - 1).to_be_bytes()].as_flattened())?;
		self.write_command(Instruction::RASET, &[calib.y.to_be_bytes(), (calib.y + HEIGHT - 1).to_be_bytes()].as_flattened())?;
		
		Ok(())
	}
	
	pub async fn draw_async(&mut self, fb: &mut Framebuffer) -> Result<()> {
		sync_perf("SPI CMD", || self.write_command(Instruction::RAMWR, &[]))?;
		
		self.a0.set_high();
		
		for chunk in fb.transfers() {
			let spi = self.spi.take().unwrap();
			
			match spi.write(chunk.len(), chunk) {
				Ok(mut transfer) => {
					transfer.wait_for_done().await;
					let (spi, _) = transfer.wait();
					self.spi = Some(spi);
				}
				Err((err, spi, _)) => {
					self.spi = Some(spi);
					
					bail!("{:?}", err);
				}
			}
		}
		
		Ok(())
	}
	
	async fn init(&mut self, calib: Axes<u16>) -> Result<()> {
		// Hard reset
		self.rst.set_high();
		self.delay.delay_ms(10).await;
		self.rst.set_low();
		self.delay.delay_ms(10).await;
		self.rst.set_high();
		
		// Init
		self.write_command(Instruction::SWRESET, &[])?;
		self.delay.delay_ms(200).await;
		self.write_command(Instruction::SLPOUT, &[])?;
		self.delay.delay_ms(200).await;
		self.write_command(Instruction::FRMCTR1, &[0x01, 0x2C, 0x2D])?;
		self.write_command(Instruction::FRMCTR2, &[0x01, 0x2C, 0x2D])?;
		self.write_command(Instruction::FRMCTR3, &[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D])?;
		self.write_command(Instruction::INVCTR, &[0x07])?;
		self.write_command(Instruction::PWCTR1, &[0xA2, 0x02, 0x84])?;
		self.write_command(Instruction::PWCTR2, &[0xC5])?;
		self.write_command(Instruction::PWCTR3, &[0x0A, 0x00])?;
		self.write_command(Instruction::PWCTR4, &[0x8A, 0x2A])?;
		self.write_command(Instruction::PWCTR5, &[0x8A, 0xEE])?;
		self.write_command(Instruction::VMCTR1, &[0x0E])?;
		self.write_command(Instruction::INVOFF, &[])?;
		self.write_command(Instruction::MADCTL, &[0x60])?;
		self.write_command(Instruction::COLMOD, &[0x05])?;
		self.write_command(Instruction::DISPON, &[])?;
		self.delay.delay_ms(200).await;
		self.apply_calib(calib)?;
		
		Ok(())
	}
	
	fn write_command(&mut self, command: Instruction, params: &[u8]) -> Result<()> {
		self.a0.set_low();
		self.write_sync(&[command as u8])?;
		
		if !params.is_empty() {
			self.a0.set_high();
			self.write_sync(params)?;
		}
		
		Ok(())
	}
	
	fn write_sync(&mut self, bytes: &[u8]) -> Result<()> {
		let spi = self.spi.take().unwrap();
		let mut cmd_buf = self.cmd_buf.take().unwrap();
		cmd_buf.fill(&bytes);
		
		match spi.write(cmd_buf.len(), cmd_buf) {
			Ok(transfer) => {
				let (spi, cmd_buf) = transfer.wait();
				self.spi = Some(spi);
				self.cmd_buf = Some(cmd_buf);
				
				Ok(())
			}
			Err((err, spi, buf)) => {
				self.spi = Some(spi);
				self.cmd_buf = Some(buf);
				
				bail!("{:?}", err)
			}
		}
	}
}

/// ST7735 instructions.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
	/// No operation
	NOP       = 0x00,
	/// Software reset
	SWRESET   = 0x01,
	/// Read display ID
	RDDID     = 0x04,
	/// Read display status
	RDDST     = 0x09,
	/// Read display power
	RDDPM     = 0x0A,
	/// Read display
	RDDMADCTL = 0x0B,
	/// Read display pixel
	RDDCOLMOD = 0x0C,
	/// Read display image
	RDDIM     = 0x0D,
	/// Read display signal
	RDDSM     = 0x0E,
	/// Sleep in
	SLPIN     = 0x10,
	/// Sleep off
	SLPOUT    = 0x11,
	/// Partial mode on
	PTLON     = 0x12,
	/// Partial mode off (normal)
	NORON     = 0x13,
	/// Display inversion off
	INVOFF    = 0x20,
	/// Display inversion on
	INVON     = 0x21,
	/// Gamma curve select
	GAMSET    = 0x26,
	/// Display off
	DISPOFF   = 0x28,
	/// Display on
	DISPON    = 0x29,
	/// Column address set
	CASET     = 0x2A,
	/// Row address set
	RASET     = 0x2B,
	/// Memory write
	RAMWR     = 0x2C,
	/// LUT (lookup table) for 4k, 65k, 262k color
	RGBSET    = 0x2D,
	/// Memory read
	RAMRD     = 0x2E,
	/// Partial start/end address set
	PTLAR     = 0x30,
	/// Tearing effect line off
	TEOFF     = 0x34,
	/// Tearing effect mode set & on
	TEON      = 0x35,
	/// Memory access data control
	MADCTL    = 0x36,
	/// Idle mode off
	IDMOFF    = 0x38,
	/// Idle mode on
	IDMON     = 0x39,
	/// Interface pixel format
	COLMOD    = 0x3A,
	/// Read ID1
	RDID1     = 0xDA,
	/// Read ID2
	RDID2     = 0xDB,
	/// Read ID3
	RDID3     = 0xDC,
	/// In normal mode (Full colors)
	FRMCTR1   = 0xB1,
	/// In idle mode (8-colors)
	FRMCTR2   = 0xB2,
	/// In partial mode (full colors)
	FRMCTR3   = 0xB3,
	/// Display inversion control
	INVCTR    = 0xB4,
	/// Power control setting
	PWCTR1    = 0xC0,
	/// Power control setting
	PWCTR2    = 0xC1,
	/// Power control setting
	PWCTR3    = 0xC2,
	/// Power control setting
	PWCTR4    = 0xC3,
	/// Power control setting
	PWCTR5    = 0xC4,
	/// VCOM control 1
	VMCTR1    = 0xC5,
	/// Set VCOM offset control
	VMOFCTR   = 0xC7,
	/// Set LCM version code
	WRID2     = 0xD1,
	/// Set customer project code
	WRID3     = 0xD2,
	/// NVM control status
	NVCTR1    = 0xD9,
	/// NVM read command
	NVCTR2    = 0xDE,
	/// NVM write command
	NVCTR3    = 0xDF,
	/// Gamma adjustment (+ polarity)
	GAMCTRP1  = 0xE0,
	/// Gamma adjustment (- polarity)
	GAMCTRN1  = 0xE1,
	/// Extension command control
	EXTCTRL   = 0xF0,
	/// VCOM 4 level control
	VCOM4L    = 0xFF,
}
