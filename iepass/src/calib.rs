#![allow(dead_code)]

#[derive(Debug, defmt::Format)]
pub struct Calib {
	pub screen_offset: Axes<u16>,
	pub analog: Axes<BiRange<u16>>,
	pub analog_deadzone: u16,
	pub touch: Axes<Range<u16>>,
}

#[derive(Debug, Copy, Clone, defmt::Format)]
pub struct Axes<T> {
	pub x: T,
	pub y: T,
}

#[derive(Debug, Copy, Clone, defmt::Format)]
pub struct Range<T> {
	pub min: T,
	pub max: T,
}

#[derive(Debug, Copy, Clone, defmt::Format)]
pub struct BiRange<T> {
	pub min: T,
	pub mid: T,
	pub max: T,
}

impl Default for Calib {
	fn default() -> Self {
		Calib { screen_offset: Axes { x: 1, y: 2 }, analog: Axes { x: BiRange { min: 1610, mid: 2716, max: 3788 }, y: BiRange { min: 1726, mid: 2722, max: 3674 } }, analog_deadzone: 6, touch: Axes { x: Range { min: 186, max: 3886 }, y: Range { min: 182, max: 3856 } } }
	}
}
