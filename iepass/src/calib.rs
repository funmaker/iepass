
#[derive(Debug)]
pub struct Calib {
	pub screen_offset: Axes<u16>,
	pub analog: Axes<BiRange<u16>>,
	pub analog_deadzone: u16,
	pub touch: Axes<Range<u16>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Axes<T> {
	pub x: T,
	pub y: T,
}

#[derive(Debug, Copy, Clone)]
pub struct Range<T> {
	pub min: T,
	pub max: T,
}

#[derive(Debug, Copy, Clone)]
pub struct BiRange<T> {
	pub min: T,
	pub mid: T,
	pub max: T,
}

impl Default for Calib {
	fn default() -> Self {
		Calib {
			screen_offset: Axes {
				x: 1,
				y: 2,
			},
			analog: Axes {
				x: BiRange {
					min: 6,
					mid: 235,
					max: 462,
				},
				y: BiRange {
					min: 5,
					mid: 235,
					max: 456,
				},
			},
			analog_deadzone: 10,
			touch: Axes {
				x: Range {
					min: 3951,
					max: 8621,
				},
				y: Range {
					min: 3727,
					max: 8329,
				},
			},
		}
	}
}
