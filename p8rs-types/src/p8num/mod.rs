use core::fmt::{Debug, Display, Formatter};
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Sub, SubAssign};

pub mod consts;

/// 16.16-bit fixed point number type.
///
/// [P8Num] uses 16 bits for integer part and 16 bits for fractional part. It can represent values from -32768.0 to 32767.9999847412109375 inclusive.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[repr(transparent)]
pub struct P8Num(i32);

impl P8Num {
	/// The smallest value that can be represented by [P8Num] (-32768.0 or 0x8000.0000).
	pub const MIN: Self = Self(i32::MIN);
	
	/// The largest value that can be represented by [P8Num] (32767.9999847412109375 or 0x7FFF.FFFF).
	pub const MAX: Self = Self(i32::MAX);
	
	/// The additive identity of [P8Num], commonly called zero (0.0 or 0x0000.0000).
	pub const ZERO: Self = Self::new(0.0);
	
	/// The multiplicative identity of [P8Num], commonly called one (1.0 or 0x0001.0000).
	pub const ONE: Self = Self::new(1.0);
	
	/// The smallest positive value of [P8Num] (0.00001525878 or 0x0000.0001).
	pub const EPSILON: Self = Self(1);
	
	/// Creates new value from f32.
	/// 
	/// Overflow is handled in a saturating manner. NaN becomes 0.
	pub const fn new(value: f32) -> Self {
		Self((value * (1 << 16) as f32) as i32)
	}
	
	/// Creates new value from f64.
	///
	/// Overflow is handled in a saturating manner. NaN becomes 0.
	pub const fn new_f64(value: f64) -> Self {
		Self((value * (1 << 16) as f64) as i32)
	}
	
	/// Constructs new [P8Num] value from raw i32 value.
	///
	/// The opposite of [Self::to_raw]. Returns [P8Num] value equal to `value` / 2^16
	pub const fn from_raw(value: i32) -> Self {
		Self(value)
	}
	
	/// Retrieves raw i32 value from [P8Num].
	///
	/// The opposite of [Self::from_raw]. Returns i32 equal to the value of `self` * 2^16
	pub const fn to_raw(self) -> i32 {
		self.0
	}
	
	/// Returns integer part.
	pub const fn integer(self) -> i16 {
		(self.0 >> 16) as i16
	}
	
	/// Returns fractional part.
	pub const fn fractional(self) -> u16 {
		(self.0 & 0xFFFF) as u16
	}
	
	/// Computes the absolute value of `self`.
	/// 
	/// # Overflow behavior
	///
	/// The absolute value of [P8Num::MIN] cannot be represented as an [P8Num]
	/// and attempting to calculate it will cause an overflow. This means
	/// that code in debug mode will trigger a panic on this case and
	/// optimized code will return [P8Num::MIN] without a panic.
	pub const fn abs(self) -> Self {
		Self(self.0.abs())
	}
	
	/// Checked absolute value. Computes `self.abs()`, returning None if `self == P8Num::MIN`.
	pub const fn checked_abs(self) -> Option<Self> {
		self.0.abs().checked_abs().map(Self)
	}
}

impl const Add for P8Num {
	type Output = P8Num;
	
	fn add(self, rhs: Self) -> P8Num {
		Self(self.0.saturating_add(rhs.0))
	}
}

impl const AddAssign for P8Num {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl const Sub for P8Num {
	type Output = P8Num;
	
	fn sub(self, rhs: Self) -> P8Num {
		Self(self.0.saturating_sub(rhs.0))
	}
}

impl const SubAssign for P8Num {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl const Mul for P8Num {
	type Output = P8Num;
	
	fn mul(self, rhs: Self) -> P8Num {
		Self(self.0.widening_mul(rhs.0).1)
	}
}

impl const MulAssign for P8Num {
	fn mul_assign(&mut self, rhs: Self) {
		*self = *self * rhs;
	}
}

impl const Div for P8Num {
	type Output = P8Num;
	
	fn div(self, rhs: Self) -> P8Num {
		Self(((self.0 as i64) << 16 / (rhs.0 as i64)) as i32)
	}
}

impl const DivAssign for P8Num {
	fn div_assign(&mut self, rhs: Self) {
		*self = *self / rhs;
	}
}

impl const Rem for P8Num {
	type Output = P8Num;
	
	fn rem(self, rhs: Self) -> P8Num {
		Self(self.0 % rhs.0)
	}
}

impl const RemAssign for P8Num {
	fn rem_assign(&mut self, rhs: Self) {
		*self = *self / rhs;
	}
}

impl const BitAnd for P8Num {
	type Output = P8Num;
	
	fn bitand(self, rhs: Self) -> P8Num {
		Self(self.0 & rhs.0)
	}
}

impl const BitAndAssign for P8Num {
	fn bitand_assign(&mut self, rhs: Self) {
		*self = *self & rhs;
	}
}

impl const BitOr for P8Num {
	type Output = P8Num;
	
	fn bitor(self, rhs: Self) -> P8Num {
		Self(self.0 | rhs.0)
	}
}

impl const BitOrAssign for P8Num {
	fn bitor_assign(&mut self, rhs: Self) {
		*self = *self | rhs;
	}
}

impl const BitXor for P8Num {
	type Output = P8Num;
	
	fn bitxor(self, rhs: Self) -> P8Num {
		Self(self.0 ^ rhs.0)
	}
}

impl const BitXorAssign for P8Num {
	fn bitxor_assign(&mut self, rhs: Self) {
		*self = *self ^ rhs;
	}
}

impl const Not for P8Num {
	type Output = P8Num;
	
	fn not(self) -> P8Num {
		Self(!self.0)
	}
}

impl const Neg for P8Num {
	type Output = P8Num;
	
	fn neg(self) -> P8Num {
		Self(-self.0)
	}
}

impl Display for P8Num {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		Display::fmt(&f64::from(*self), f)
	}
}

impl Debug for P8Num {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "0x{:04X}.{:04X}", self.integer(), self.fractional())
	}
}

impl const From<P8Num> for f32 {
	fn from(value: P8Num) -> f32 {
		value.0 as f32 / (1 << 16) as f32
	}
}

impl const From<P8Num> for f64 {
	fn from(value: P8Num) -> f64 {
		value.0 as f64 / (1 << 16) as f64
	}
}

impl const From<i16> for P8Num {
	fn from(integer: i16) -> Self {
		P8Num((integer as i32) << 16)
	}
}

impl const From<i8> for P8Num {
	fn from(integer: i8) -> Self {
		P8Num((integer as i32) << 16)
	}
}

impl const From<u8> for P8Num {
	fn from(integer: u8) -> Self {
		P8Num((integer as i32) << 16)
	}
}

impl const TryFrom<f32> for P8Num {
	type Error = TryFromError;
	
	fn try_from(value: f32) -> Result<Self, Self::Error> {
		if !value.is_finite() || value < P8Num::MIN.into() || value > P8Num::MAX.into() {
			return Err(TryFromError::OutOfRange)
		}
		
		Ok(P8Num::new(value))
	}
}

impl const TryFrom<f64> for P8Num {
	type Error = TryFromError;
	
	fn try_from(value: f64) -> Result<Self, Self::Error> {
		if !value.is_finite() || value < P8Num::MIN.into() || value > P8Num::MAX.into() {
			return Err(TryFromError::OutOfRange)
		}
		
		Ok(P8Num::new_f64(value))
	}
}

/// The error type returned when a checked conversion fails.
#[non_exhaustive]
pub enum TryFromError {
	/// The provided value is outside the range of representable values or is NaN.
	OutOfRange,
}
