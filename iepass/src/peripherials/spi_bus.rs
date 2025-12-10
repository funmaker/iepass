use core::ops::Deref;
use anyhow::{Result, anyhow};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use esp_hal::{dma_buffers, Async};
use esp_hal::dma::{DmaChannelFor, DmaRxBuf, DmaTxBuf};
use esp_hal::gpio::{InputPin, OutputPin};
use esp_hal::i2s::AnyI2s;
use esp_hal::spi::master::{Config, Instance, Spi, SpiDmaBus};
use esp_hal::time::Rate;

type SpiBusInner<'d> = Mutex<CriticalSectionRawMutex, SpiDmaBus<'d, Async>>;

#[derive(Debug)]
pub struct SpiBus<'d> {
	inner: SpiBusInner<'d>,
}

impl<'d> Deref for SpiBus<'d> {
	type Target = SpiBusInner<'d>;
	
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'d> SpiBus<'d> {
	pub fn new(spi: impl Instance + 'd,
	           mosi: impl OutputPin + 'd,
	           sck: impl OutputPin + 'd,
	           miso: impl InputPin + 'd,
	           dma: impl DmaChannelFor<AnyI2s<'d>> + 'd)
	           -> Result<Self> {
		let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(8000);
		let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).map_err(|err| anyhow!("{err:?}"))?;
		let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).map_err(|err| anyhow!("{err:?}"))?;
		
		let spi = Spi::new(
			spi,
			Config::default()
				.with_frequency(Rate::from_khz(1000)),
		)?.with_mosi(mosi)
		  .with_sck(sck)
		  .with_miso(miso)
		  .with_dma(dma)
		  .with_buffers(dma_rx_buf, dma_tx_buf)
		  .into_async();
		
		Ok(Self {
			inner: Mutex::new(spi),
		})
	}
}
