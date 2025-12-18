use defmt::Format;
use esp_hal::analog::adc::AdcChannel;
use esp_hal::gpio::AnalogPin;

use crate::peripherials::{Debounce, Analog};

pub struct Controller<'d, PinAnalX, PinAnalY>
where PinAnalX: AdcChannel,
      PinAnalY: AdcChannel {
	select_btn: Debounce<'d>,
	start_btn: Debounce<'d>,
	x_btn: Debounce<'d>,
	y_btn: Debounce<'d>,
	a_btn: Debounce<'d>,
	b_btn: Debounce<'d>,
	analog_btn: Debounce<'d>,
	analog: Analog<'d, PinAnalX, PinAnalY>,
}

#[derive(Debug, Format)]
pub struct ControllerState {
	pub select_btn: bool,
	pub start_btn: bool,
	pub x_btn: bool,
	pub y_btn: bool,
	pub a_btn: bool,
	pub b_btn: bool,
	pub analog_btn: bool,
	pub analog: (i16, i16),
	pub up: bool,
	pub down: bool,
	pub left: bool,
	pub right: bool,
}

impl<'d, PinAnalX, PinAnalY> Controller<'d, PinAnalX, PinAnalY>
where PinAnalX: AdcChannel + AnalogPin + 'static,
      PinAnalY: AdcChannel + AnalogPin + 'static {
	pub fn new(select_btn: Debounce<'d>,
	           start_btn: Debounce<'d>,
	           x_btn: Debounce<'d>,
	           y_btn: Debounce<'d>,
	           a_btn: Debounce<'d>,
	           b_btn: Debounce<'d>,
	           analog_btn: Debounce<'d>,
	           analog: Analog<'d, PinAnalX, PinAnalY>) -> Self {
		Self {
			select_btn,
			start_btn,
			x_btn,
			y_btn,
			a_btn,
			b_btn,
			analog_btn,
			analog,
		}
	}
	
	pub fn get_state(&mut self) -> ControllerState {
		let (analog_x, analog_y) = self.analog.read(100);
		
		ControllerState {
			select_btn: self.select_btn.is_high(),
			start_btn: self.start_btn.is_high(),
			x_btn: self.x_btn.is_high(),
			y_btn: self.y_btn.is_high(),
			a_btn: self.a_btn.is_high(),
			b_btn: self.b_btn.is_high(),
			analog_btn: self.analog_btn.is_high(),
			analog: (analog_x, analog_y),
			up: analog_y < -75,
			down: analog_y > 75,
			left: analog_x < -75,
			right: analog_x > 75,
		}
	}
}
