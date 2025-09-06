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
		Calib { screen_offset: Axes { x: 1, y: 2 }, analog: Axes { x: BiRange { min: 1783, mid: 2854, max: 3849 }, y: BiRange { min: 1807, mid: 2863, max: 3863 } }, analog_deadzone: 12, touch: Axes { x: Range { min: 192, max: 3854 }, y: Range { min: 321, max: 3629 } } }
	}
}
