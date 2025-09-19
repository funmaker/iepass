
pub struct P8Num {
	integer: i16,
	fraction: u16,
}

impl P8Num {
	pub const MIN: Self = Self { integer: i16::MIN, fraction: u16::MIN };
	pub const MAX: Self = Self { integer: i16::MAX, fraction: u16::MAX };
	pub const ZERO: Self = Self { integer: 0, fraction: 0 };
	pub const ONE: Self = Self { integer: 1, fraction: 0 };
	
	pub const fn new(value: f32) -> Self {
		if value.is_nan() {
			Self::ZERO
		} else if value < Self::MIN.into() {
			Self::MIN
		} else if value > Self::MAX.into() {
			Self::MAX
		} else {
			P8Num {
				integer: value as i16,
				fraction: ((value - value.floor()) * (u16::MAX as f32 + 1.0)).trunc() as u16,
			}
		}
	}
}

impl const From<P8Num> for f32 {
	fn from(value: P8Num) -> Self {
		value.integer as f32 + value.fraction as f32 / (u16::MAX as f32 + 1.0)
	}
}

impl const From<P8Num> for f64 {
	fn from(value: P8Num) -> Self {
		value.integer as f64 + value.fraction as f64 / (u16::MAX as f64 + 1.0)
	}
}

impl const From<i16> for P8Num {
	fn from(integer: i16) -> Self {
		P8Num { integer, fraction: 0 }
	}
}

impl const From<i8> for P8Num {
	fn from(integer: i8) -> Self {
		P8Num { integer: integer as i16, fraction: 0 }
	}
}

impl const From<u8> for P8Num {
	fn from(integer: u8) -> Self {
		P8Num { integer: integer as i16, fraction: 0 }
	}
}

impl const TryFrom<f32> for P8Num {
	type Error = TryFromError;
	
	fn try_from(value: f32) -> Result<Self, Self::Error> {
		if !value.is_finite() || value < P8Num::MIN.into() || value > P8Num::MAX.into() {
			return Err(TryFromError::OutOfRange)
		}
		
		Ok(P8Num {
			integer: value.trunc() as i16,
			fraction: ((value - value.floor()) * (u16::MAX as f32 + 1.0)).trunc() as u16,
		})
	}
}

impl const TryFrom<f64> for P8Num {
	type Error = TryFromError;
	
	fn try_from(value: f64) -> Result<Self, Self::Error> {
		if !value.is_finite() || value < P8Num::MIN.into() || value > P8Num::MAX.into() {
			return Err(TryFromError::OutOfRange)
		}
		
		Ok(P8Num {
			integer: value.trunc() as i16,
			fraction: ((value - value.floor()) * (u16::MAX as f64 + 1.0)).trunc() as u16,
		})
	}
}

#[non_exhaustive]
pub enum TryFromError {
	OutOfRange,
}
