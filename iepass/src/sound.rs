use esp_idf_svc::hal::gpio::{AnyIOPin, InputPin, OutputPin};
use esp_idf_svc::hal::i2s::{I2s, I2sDriver, I2sTx};
use esp_idf_svc::hal::i2s::config::{Config, DataBitWidth, SlotMode, StdClkConfig, StdConfig, StdGpioConfig, StdSlotConfig, StdSlotMask};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::sys::EspError;

pub struct Sound<'d> {
	driver: I2sDriver<'d, I2sTx>,
}

impl<'d> Sound<'d> {
	pub fn new(
		i2s: impl Peripheral<P = impl I2s> + 'd,
		bclk: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
		dout: impl Peripheral<P = impl OutputPin> + 'd,
		ws: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
	) -> Result<Self, EspError> {
		let mut driver = I2sDriver::new_std_tx(
			i2s,
			&StdConfig::new(
				Config::default().frames_per_buffer(37),
				StdClkConfig::from_sample_rate_hz(44100),
				StdSlotConfig::msb_slot_default(DataBitWidth::Bits16, SlotMode::Stereo).slot_mode_mask(SlotMode::Mono, StdSlotMask::Both),
				StdGpioConfig::default(),
			),
			bclk,
			dout,
			AnyIOPin::none(),
			ws,
		)?;
		
		Ok(Self { driver })
	}
	
	pub fn play(&mut self, data: &[u8]) -> Result<(), EspError> {
		
		self.driver.tx_enable()?;
		self.driver.write_all(data, 1000000)?;
		self.driver.tx_disable()?;
		
		Ok(())
	}
}
