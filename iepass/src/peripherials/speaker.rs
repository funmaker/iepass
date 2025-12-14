use core::error::Error;
use anyhow::{anyhow, Result};
use esp_hal::i2s::master::asynch::I2sWriteDmaTransferAsync;
use esp_hal::i2s::master::{DataFormat, I2s, Instance, Config, Channels};
use esp_hal::i2s::AnyI2s;
use esp_hal::time::Rate;
use esp_hal::gpio::OutputPin;
use esp_hal::dma::{DmaChannelFor};
use esp_hal::{dma_tx_buffer};

const BUFFER_SIZE: usize = 8184;

pub struct Speaker<'d> {
	pub transfer: I2sWriteDmaTransferAsync<'d, &'static mut [u8]>,
}

impl<'d> Speaker<'d> {
	pub fn new(i2s: impl Instance + 'd,
	           blck: impl OutputPin + 'd,
	           ws: impl OutputPin + 'd,
	           dout: impl OutputPin + 'd,
	           dma_channel: impl DmaChannelFor<AnyI2s<'d>>)
	           -> Result<Self> {
		let buffer = dma_tx_buffer!(BUFFER_SIZE).map_err(|err| anyhow!("{:?}", err))?;
		let (descriptors, buffer) = buffer.split();
		
		let i2s = I2s::new(
			i2s,
			dma_channel,
			Config::new_tdm_philips()
				.with_sample_rate(Rate::from_hz(44100))
				.with_channels(Channels::MONO)
				.with_data_format(DataFormat::Data16Channel16)
		);
		
		let transfer = i2s
			.unwrap()
			.into_async()
			.i2s_tx
			.with_bclk(blck)
			.with_ws(ws)
			.with_dout(dout)
			.build(descriptors)
			.write_dma_circular_async(buffer)
			.map_err(|err| anyhow!("{:?}", err))?;
		
		Ok(Speaker { transfer })
	}
	
	pub async fn play(&mut self, mut reader: impl embedded_io::Read<Error = impl Error + Send + Sync>) -> Result<()> {
		let mut result = Ok(0);
		
		loop {
			self.transfer.push_with(|chunk| {
				result = reader.read(chunk);
				result.as_ref().copied().unwrap_or(0)
			}).await
			  .map_err(|err| anyhow!("{:?}", err))?;
			
			if let Ok(0) = result {
				return Ok(());
			}
		}
	}
	
	pub async fn reset(&mut self) -> Result<()> {
		let mut written = 0;
		
		while written < BUFFER_SIZE {
			self.transfer.push_with(|chunk| {
				chunk.fill(0);
				written += chunk.len();
				chunk.len()
			}).await
			  .map_err(|err| anyhow!("{:?}", err))?;
		}
		
		Ok(())
	}
}
