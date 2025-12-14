use core::fmt::{Debug, Formatter};
use esp_hal::analog::adc::AdcChannel;
use esp_hal::gpio::AnalogPin;
use p8rs::vm::Callbacks;

use crate::peripherials::controller::Controller;

pub struct IepassCallbacks<'d, PinAnalX, PinAnalY>
where PinAnalX: AdcChannel,
      PinAnalY: AdcChannel {
	controller: Controller<'d, PinAnalX, PinAnalY>
}

impl<'d, PinAnalX, PinAnalY> IepassCallbacks<'d, PinAnalX, PinAnalY>
where PinAnalX: AdcChannel,
      PinAnalY: AdcChannel {
	pub fn new(controller: Controller<'d, PinAnalX, PinAnalY>) -> Self {
		Self { controller }
	}
}

impl<PinAnalX, PinAnalY> Debug for IepassCallbacks<'_, PinAnalX, PinAnalY>
where PinAnalX: AdcChannel,
      PinAnalY: AdcChannel {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "IepassCallbacks")
	}
}

impl<PinAnalX, PinAnalY> Callbacks for IepassCallbacks<'_, PinAnalX, PinAnalY>
where PinAnalX: AdcChannel + AnalogPin + 'static,
      PinAnalY: AdcChannel + AnalogPin + 'static {
	fn get_buttons(&mut self) -> [u8; 8] {
		let state = self.controller.get_state();
		let mut ret = 0;
		
		if state.left { ret |= 1 << 0; }
		if state.right { ret |= 1 << 1; }
		if state.up { ret |= 1 << 2; }
		if state.down { ret |= 1 << 3; }
		if state.a_btn { ret |= 1 << 4; }
		if state.b_btn { ret |= 1 << 5; }
		if state.x_btn { ret |= 1 << 4; }
		if state.y_btn { ret |= 1 << 5; }
		if state.select_btn { ret |= 1 << 6; }
		if state.start_btn { ret |= 1 << 6; }
		
		[ret, 0, 0, 0, 0, 0, 0, 0]
	}
}
