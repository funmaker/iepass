use core::fmt::{Debug, Display, Formatter};
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign};

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
	
	/// Creates new value from `f32`.
	/// 
	/// Overflow is handled in a saturating manner. NaN becomes 0.
	/// 
	/// # Examples
	/// 
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::new(1.0), P8Num::from_raw(0x0001_0000));
	/// assert_eq!(P8Num::new(0.5), P8Num::from_raw(0x0000_8000));
	/// assert_eq!(P8Num::new(-0.5), P8Num::from_raw(-0x0000_8000));
	/// assert_eq!(P8Num::new(4660.337890625), P8Num::from_raw(0x1234_5680));
	/// assert_eq!(P8Num::new(100_000.0), P8Num::MAX);
	/// assert_eq!(P8Num::new(f32::INFINITY), P8Num::MAX);
	/// assert_eq!(P8Num::new(-100_000.0), P8Num::MIN);
	/// assert_eq!(P8Num::new(f32::NEG_INFINITY), P8Num::MIN);
	/// assert_eq!(P8Num::new(f32::NAN), P8Num::ZERO);
	/// ```
	pub const fn new(value: f32) -> Self {
		Self((value * (1 << 16) as f32) as i32)
	}
	
	/// Creates new value from `f64`.
	///
	/// Overflow is handled in a saturating manner. NaN becomes 0.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new_f64(1.0), P8Num::from_raw(0x0001_0000));
	/// assert_eq!(P8Num::new_f64(0.5), P8Num::from_raw(0x0000_8000));
	/// assert_eq!(P8Num::new_f64(-0.5), P8Num::from_raw(-0x0000_8000));
	/// assert_eq!(P8Num::new_f64(4660.3377685546875), P8Num::from_raw(0x1234_5678));
	/// assert_eq!(P8Num::new_f64(100_000.0), P8Num::MAX);
	/// assert_eq!(P8Num::new_f64(f64::INFINITY), P8Num::MAX);
	/// assert_eq!(P8Num::new_f64(-100_000.0), P8Num::MIN);
	/// assert_eq!(P8Num::new_f64(f64::NEG_INFINITY), P8Num::MIN);
	/// assert_eq!(P8Num::new_f64(f64::NAN), P8Num::ZERO);
	/// ```
	pub const fn new_f64(value: f64) -> Self {
		Self((value * (1 << 16) as f64) as i32)
	}
	
	/// Parses an integer from an P8SCII slice with decimal digits.
	///
	/// The characters are expected to be an optional
	///  `+` or `-` 
	/// sign followed by only digits. Leading and trailing non-digit characters (including
	/// whitespace) represent an error. Underscores (which are accepted in Rust literals)
	/// also represent an error.
	///
	/// # Examples
	///
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_p8scii(b"+10"), Ok(P8Num::new(10.0)));
	/// ```
	/// Trailing space returns error:
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!(P8Num::from_p8scii(b"1 ").is_err());
	/// ```
	#[inline]
	pub const fn from_p8scii(_src: &[u8]) -> Result<Self, ()> {
		unimplemented!()
	}
	
	/// Parses an integer from an ASCII-byte slice with digits in a given base.
	///
	/// The characters are expected to be an optional
	///  `+` or `-` 
	/// sign followed by only digits. Leading and trailing non-digit characters (including
	/// whitespace) represent an error. Underscores (which are accepted in Rust literals)
	/// also represent an error.
	///
	/// Digits are a subset of these characters, depending on `radix`:
	/// * `0-9`
	/// * `a-z`
	/// * `A-Z`
	///
	/// # Panics
	///
	/// This function panics if `radix` is not in the range from 2 to 36.
	///
	/// # Examples
	///
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_p8scii_radix(b"A", 16), Ok(P8Num::new(10.0)));
	/// ```
	/// Trailing space returns error:
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!(P8Num::from_p8scii_radix(b"1 ", 10).is_err());
	/// ```
	#[inline]
	pub const fn from_p8scii_radix(_src: &[u8], _radix: u32) -> Result<Self, ()> {
		unimplemented!()
	}
	
	/// Constructs new value from raw i32 value.
	///
	/// The opposite of [Self::to_raw]. Returns [P8Num] value equal to `value` / 2^16
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x0001_0000), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(-0x0001_0000), P8Num::new(-1.0));
	/// assert_eq!(P8Num::from_raw(0x0000_8000), P8Num::new(0.5));
	/// assert_eq!(P8Num::from_raw(-0x0000_8000), P8Num::new(-0.5));
	/// assert_eq!(P8Num::from_raw(0x1234_5678), P8Num::new_f64(4660.3377685546875));
	/// ```
	pub const fn from_raw(value: i32) -> Self {
		Self(value)
	}
	
	/// Retrieves raw i32 value from `self`.
	///
	/// The opposite of [Self::from_raw]. Returns i32 equal to the value of `self` * 2^16
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(1.0).to_raw(), 0x0001_0000);
	/// assert_eq!(P8Num::new(-1.0).to_raw(), -0x0001_0000);
	/// assert_eq!(P8Num::new(0.5).to_raw(), 0x0000_8000);
	/// assert_eq!(P8Num::new(-0.5).to_raw(), -0x0000_8000);
	/// assert_eq!(P8Num::new_f64(4660.3377685546875).to_raw(), 0x1234_5678);
	/// ```
	pub const fn to_raw(self) -> i32 {
		self.0
	}
	
	/// Returns integer part, rounded down.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(1.0).integer(), 1);
	/// assert_eq!(P8Num::new(-1.0).integer(), -1);
	/// assert_eq!(P8Num::new(0.5).integer(), 0);
	/// assert_eq!(P8Num::new(-0.5).integer(), -1);
	/// assert_eq!(P8Num::from_raw(0x1234_5678).integer(), 0x1234);
	/// ```
	pub const fn integer(self) -> i16 {
		(self.0 >> 16) as i16
	}
	
	/// Returns fractional part.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(1.0).fractional(), 0);
	/// assert_eq!(P8Num::new(-1.0).fractional(), 0);
	/// assert_eq!(P8Num::new(0.5).fractional(), 0x8000);
	/// assert_eq!(P8Num::new(-0.5).fractional(), 0x8000);
	/// assert_eq!(P8Num::from_raw(0x1234_5678).fractional(), 0x5678);
	/// ```
	pub const fn fractional(self) -> u16 {
		(self.0 & 0xFFFF) as u16
	}
	
	/// Computes the absolute value of `self`.
	///
	/// # Overflow behavior
	///
	/// The absolute value of `P8Num::MIN` cannot be represented as an `i32`, and attempting
	/// to calculate it will cause an overflow. This means that code in debug mode will trigger
	/// a panic on this case and optimized code will return `P8Num::MIN` without a panic. If you
	/// do not want this behavior, consider using [`unsigned_abs`](Self::unsigned_abs) instead.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(10.0).abs(), P8Num::new(10.0));
	/// assert_eq!(P8Num::new(-10.0).abs(), P8Num::new(10.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn abs(self) -> Self {
		Self(self.0.abs())
	}
	
	/// Returns a number representing sign of `self`.
	///
	///  - `0` if the number is zero
	///  - `1` if the number is positive
	///  - `-1` if the number is negative
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(10.0).signum(), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(0.0).signum(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(-10.0).signum(), P8Num::new(-1.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn signum(self) -> Self {
		Self(self.0.signum() << 16)
	}
	
	/// Returns `true` if `self` is positive and `false` if the number is zero or
	/// negative.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!(P8Num::new(10.0).is_positive());
	/// assert!(!P8Num::new(0.0).is_positive());
	/// assert!(!P8Num::new(-10.0).is_positive());
	/// ```
	#[must_use]
	#[inline(always)]
	pub const fn is_positive(self) -> bool {
		self.0.is_positive()
	}
	
	/// Returns `true` if `self` is negative and `false` if the number is zero or
	/// positive.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!(P8Num::new(-10.0).is_negative());
	/// assert!(!P8Num::new(0.0).is_negative());
	/// assert!(!P8Num::new(10.0).is_negative());
	/// ```
	#[must_use]
	#[inline(always)]
	pub const fn is_negative(self) -> bool {
		self.0.is_negative()
	}
	
	/// Calculates the midpoint (average) between `self` and `rhs`.
	///
	/// `midpoint(a, b)` is `(a + b) / 2` as if it were performed in a
	/// sufficiently-large signed integral type. This implies that the result is
	/// always rounded towards zero and that no overflow will ever occur.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(1.0).midpoint(P8Num::new(4.0)), P8Num::new(2.5));
	/// assert_eq!(P8Num::new(-5.5).midpoint(P8Num::new(8.0)), P8Num::new(1.25));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn midpoint(self, rhs: Self) -> Self {
		Self(self.0.midpoint(rhs.0))
	}
	
	/// Calculates Euclidean division, the matching method for `rem_euclid`.
	///
	/// This computes the integer `n` such that
	/// `self = n * rhs + self.rem_euclid(rhs)`.
	/// In other words, the result is `self / rhs` rounded to the integer `n`
	/// such that `self >= n * rhs`.
	///
	/// # Precision
	///
	/// The result of this operation is guaranteed to be the rounded
	/// infinite-precision result.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero or the operation would result in overflow.
	/// This behavior is not affected by the `overflow-checks` flag.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).div_euclid(P8Num::new(4.0)), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(-7.0).div_euclid(P8Num::new(4.0)), P8Num::new(-2.0));
	/// assert_eq!(P8Num::new(7.0).div_euclid(P8Num::new(-4.0)), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-7.0).div_euclid(P8Num::new(-4.0)), P8Num::new(2.0));
	/// assert_eq!(P8Num::new(10.0).div_euclid(P8Num::new(0.25)), P8Num::new(40.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	#[track_caller]
	pub const fn div_euclid(self, rhs: Self) -> Self {
		self.checked_div_euclid(rhs).unwrap()
	}
	
	/// Calculates the least nonnegative remainder of `self (mod rhs)`.
	///
	/// This is done as if by the Euclidean division algorithm -- given
	/// `r = self.rem_euclid(rhs)`, the result satisfies
	/// `self = rhs * self.div_euclid(rhs) + r` and `0 <= r < abs(rhs)`.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero or the operation would result in overflow.
	/// This behavior is not affected by the `overflow-checks` flag.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).rem_euclid(P8Num::new(4.0)), P8Num::new(3.0));
	/// assert_eq!(P8Num::new(-7.0).rem_euclid(P8Num::new(4.0)), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(7.0).rem_euclid(P8Num::new(-4.0)), P8Num::new(3.0));
	/// assert_eq!(P8Num::new(-7.0).rem_euclid(P8Num::new(-4.0)), P8Num::new(1.0));
	/// ```
	///
	/// This will panic:
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// let _ = P8Num::MIN.rem_euclid(-P8Num::EPSILON);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	#[track_caller]
	pub const fn rem_euclid(self, rhs: Self) -> Self {
		self.checked_rem_euclid(rhs).unwrap()
	}
	
	/// Raises self to the power of `exp`, using exponentiation by squaring.
	///
	/// # Examples
	///
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(3.0).pow(P8Num::new(4.0)), P8Num::new(81.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn pow(self, mut _exp: Self) -> Self {
		unimplemented!()
	}
	
	/// Returns the number of ones in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::from_raw(0x0001_0000).count_ones(), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(0x0000_1111).count_ones(), P8Num::new(4.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn count_ones(self) -> Self {
		Self::from(self.0.count_ones() as i16)
	}
	
	/// Returns the number of zeros in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::new(0.0).count_zeros(), P8Num::new(32.0));
	/// assert_eq!(P8Num::new(-1.0).count_zeros(), P8Num::new(16.0));
	/// assert_eq!((-P8Num::EPSILON).count_zeros(), P8Num::new(0.0));
	/// assert_eq!(P8Num::MAX.count_zeros(), P8Num::new(1.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn count_zeros(self) -> Self {
		Self::from(self.0.count_zeros() as i16)
	}
	
	/// Returns the number of leading zeros in the binary representation of `self`.
	///
	/// Depending on what you're doing with the value, you might also be interested in the
	/// [`ilog2`] function which returns a consistent number, even if the type widens.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).leading_zeros(), P8Num::new(32.0));
	/// assert_eq!(P8Num::new(1.0).leading_zeros(), P8Num::new(15.0));
	/// assert_eq!((-P8Num::EPSILON).leading_zeros(), P8Num::new(0.0));
	/// assert_eq!(P8Num::MAX.leading_zeros(), P8Num::new(1.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn leading_zeros(self) -> Self {
		Self::from(self.0.leading_zeros() as i16)
	}
	
	/// Returns the number of trailing zeros in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).trailing_zeros(), P8Num::new(32.0));
	/// assert_eq!(P8Num::new(1.0).trailing_zeros(), P8Num::new(16.0));
	/// assert_eq!((-P8Num::EPSILON).trailing_zeros(), P8Num::new(0.0));
	/// assert_eq!(P8Num::MAX.trailing_zeros(), P8Num::new(0.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn trailing_zeros(self) -> Self {
		Self::from(self.0.trailing_zeros() as i16)
	}
	
	/// Returns the number of leading ones in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).leading_ones(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(-1.0).leading_ones(), P8Num::new(16.0));
	/// assert_eq!((-P8Num::EPSILON).leading_ones(), P8Num::new(32.0));
	/// assert_eq!(P8Num::MAX.leading_ones(), P8Num::new(0.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn leading_ones(self) -> Self {
		Self::from(self.0.leading_ones() as i16)
	}
	
	/// Returns the number of trailing ones in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).trailing_ones(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(1.0).trailing_ones(), P8Num::new(0.0));
	/// assert_eq!((-P8Num::EPSILON).trailing_ones(), P8Num::new(32.0));
	/// assert_eq!(P8Num::MAX.trailing_ones(), P8Num::new(31.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn trailing_ones(self) -> Self {
		Self::from(self.0.trailing_ones() as i16)
	}
	
	/// Shifts the bits to the left by a specified amount, `n`,
	/// wrapping the truncated bits to the end of the resulting integer.
	///
	/// Please note this isn't the same operation as the `<<` shifting operator!
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x0001_0000).rotate_left(8), P8Num::from_raw(0x0100_0000));
	/// assert_eq!(P8Num::from_raw(0x0000_8000).rotate_left(8), P8Num::from_raw(0x0080_0000));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).rotate_left(8), P8Num::from_raw(0x3456_7812));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn rotate_left(self, n: u32) -> Self {
		Self(self.0.rotate_left(n))
	}
	
	/// Shifts the bits to the right by a specified amount, `n`,
	/// wrapping the truncated bits to the beginning of the resulting
	/// integer.
	///
	/// Please note this isn't the same operation as the `>>` shifting operator!
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x0001_0000).rotate_right(8), P8Num::from_raw(0x0000_0100));
	/// assert_eq!(P8Num::from_raw(0x0000_8000).rotate_right(8), P8Num::from_raw(0x0000_0080));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).rotate_right(8), P8Num::from_raw(0x7812_3456));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn rotate_right(self, n: u32) -> Self {
		Self(self.0.rotate_right(n))
	}
	
	/// Reverses the byte order of the integer.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x1234_5678).swap_bytes(), P8Num::from_raw(0x7856_3412));
	/// assert_eq!(P8Num::from_raw(0x0000_0000).swap_bytes(), P8Num::from_raw(0x0000_0000));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn swap_bytes(self) -> Self {
		Self(self.0.swap_bytes())
	}
	
	/// Reverses the order of bits in the integer. The least significant bit becomes the most significant bit,
	///                 second least-significant bit becomes second most-significant bit, etc.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x1234_5678).reverse_bits(), P8Num::from_raw(0x1e6a2c48));
	/// assert_eq!(P8Num::from_raw(0x0000_0000).reverse_bits(), P8Num::from_raw(0x0000_0000));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn reverse_bits(self) -> Self {
		Self(self.0.reverse_bits())
	}
	
	/// Checked integer addition. Computes `self + rhs`, returning `None`
	/// if overflow occurred.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!((P8Num::MAX - P8Num::new(2.0)).checked_add(P8Num::new(1.0)), Some(P8Num::MAX - P8Num::new(1.0)));
	/// assert_eq!((P8Num::MAX - P8Num::new(2.0)).checked_add(P8Num::new(3.0)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_add(self, rhs: Self) -> Option<Self> {
		self.0.checked_add(rhs.0).map(Self)
	}
	
	/// Checked integer subtraction. Computes `self - rhs`, returning `None` if
	/// overflow occurred.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!((P8Num::MIN + P8Num::new(2.0)).checked_sub(P8Num::new(1.0)), Some(P8Num::MIN + P8Num::new(1.0)));
	/// assert_eq!((P8Num::MIN + P8Num::new(2.0)).checked_sub(P8Num::new(3.0)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
		self.0.checked_sub(rhs.0).map(Self)
	}
	
	/// Checked integer multiplication. Computes `self * rhs`, returning `None` if
	/// overflow occurred.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::MAX.checked_mul(P8Num::new(1.0)), Some(P8Num::MAX));
	/// assert_eq!(P8Num::MAX.checked_mul(P8Num::new(2.0)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
		try_into_some(((self.0 as i64) * (rhs.0 as i64)) >> 16)
			.map(Self)
	}
	
	/// Checked integer division. Computes `self / rhs`, returning `None` if `rhs == 0`
	/// or the division results in overflow.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).checked_div(P8Num::new(4.0)), Some(P8Num::new(1.75)));
	/// assert_eq!(P8Num::new(-7.0).checked_div(P8Num::new(4.0)), Some(P8Num::new(-1.75)));
	/// assert_eq!(P8Num::new(7.0).checked_div(P8Num::new(-4.0)), Some(P8Num::new(-1.75)));
	/// assert_eq!(P8Num::new(-7.0).checked_div(P8Num::new(-4.0)), Some(P8Num::new(1.75)));
	/// assert_eq!(P8Num::new(10.0).checked_div(P8Num::new(0.25)), Some(P8Num::new(40.0)));
	/// assert_eq!((P8Num::MIN + P8Num::EPSILON).checked_div(P8Num::new(-1.0)), Some(P8Num::MAX));
	/// assert_eq!(P8Num::new(30000.0).checked_div(P8Num::new(0.5)), None);
	/// assert_eq!(P8Num::MIN.checked_div(P8Num::new(-1.0)), None);
	/// assert_eq!(P8Num::new(1.0).checked_div(P8Num::new(0.0)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_div(self, rhs: Self) -> Option<Self> {
		if rhs.0 == 0 {
			return None;
		}
		
		try_into_some(((self.0 as i64) << 16) / (rhs.0 as i64))
			.map(Self)
	}
	
	/// Checked Euclidean division. Computes `self.div_euclid(rhs)`,
	/// returning `None` if `rhs == 0` or the division results in overflow.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).checked_div_euclid(P8Num::new(4.0)), Some(P8Num::new(1.0)));
	/// assert_eq!(P8Num::new(-7.0).checked_div_euclid(P8Num::new(4.0)), Some(P8Num::new(-2.0)));
	/// assert_eq!(P8Num::new(7.0).checked_div_euclid(P8Num::new(-4.0)), Some(P8Num::new(-1.0)));
	/// assert_eq!(P8Num::new(-7.0).checked_div_euclid(P8Num::new(-4.0)), Some(P8Num::new(2.0)));
	/// assert_eq!(P8Num::new(10.5).checked_div_euclid(P8Num::new(0.25)), Some(P8Num::new(42.0)));
	/// assert_eq!(P8Num::new(30000.0).checked_div_euclid(P8Num::new(0.5)), None);
	/// assert_eq!(P8Num::MIN.checked_div_euclid(P8Num::new(-1.0)), None);
	/// assert_eq!(P8Num::new(1.0).checked_div_euclid(P8Num::new(0.0)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_div_euclid(self, rhs: Self) -> Option<Self> {
		if rhs.0 == 0 {
			return None;
		}
		
		try_into_some((self.0 as i64).div_euclid(rhs.0 as i64) << 16)
			.map(Self)
	}
	
	/// Checked integer remainder. Computes `self % rhs`, returning `None` if
	/// `rhs == 0` or the division results in overflow.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).checked_rem(P8Num::new(4.0)), Some(P8Num::new(3.0)));
	/// assert_eq!(P8Num::new(-7.0).checked_rem(P8Num::new(4.0)), Some(P8Num::new(-3.0)));
	/// assert_eq!(P8Num::new(7.0).checked_rem(P8Num::new(-4.0)), Some(P8Num::new(3.0)));
	/// assert_eq!(P8Num::new(-7.0).checked_rem(P8Num::new(-4.0)), Some(P8Num::new(-3.0)));
	/// assert_eq!(P8Num::new(7.0).checked_rem(P8Num::new(0.0)), None);
	/// assert_eq!(P8Num::MIN.checked_rem(-P8Num::EPSILON), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_rem(self, rhs: Self) -> Option<Self> {
		self.0.checked_rem(rhs.0).map(Self)
	}
	
	/// Checked Euclidean remainder. Computes `self.rem_euclid(rhs)`, returning `None`
	/// if `rhs == 0` or the division results in overflow.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).checked_rem_euclid(P8Num::new(4.0)), Some(P8Num::new(3.0)));
	/// assert_eq!(P8Num::new(-7.0).checked_rem_euclid(P8Num::new(4.0)), Some(P8Num::new(1.0)));
	/// assert_eq!(P8Num::new(7.0).checked_rem_euclid(P8Num::new(-4.0)), Some(P8Num::new(3.0)));
	/// assert_eq!(P8Num::new(-7.0).checked_rem_euclid(P8Num::new(-4.0)), Some(P8Num::new(1.0)));
	/// assert_eq!(P8Num::new(7.5).checked_rem_euclid(P8Num::new(4.0)), Some(P8Num::new(3.5)));
	/// assert_eq!(P8Num::new(7.0).checked_rem_euclid(P8Num::new(0.0)), None);
	/// assert_eq!(P8Num::MIN.checked_rem_euclid(-P8Num::EPSILON), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_rem_euclid(self, rhs: Self) -> Option<Self> {
		self.0.checked_rem_euclid(rhs.0).map(Self)
	}
	
	/// Checked negation. Computes `-self`, returning `None` if `self == MIN`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(7.0).checked_neg(), Some(P8Num::new(-7.0)));
	/// assert_eq!(P8Num::MIN.checked_neg(), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_neg(self) -> Option<Self> {
		self.0.checked_neg().map(Self)
	}
	
	/// Checked shift left. Computes `self << rhs`, returning `None` if `rhs` is larger
	/// than or equal to the number of bits in `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(1.0).checked_shl(4), Some(P8Num::new(16.0)));
	/// assert_eq!(P8Num::new(1.0).checked_shl(129), None);
	/// assert_eq!(P8Num::new(10.0).checked_shl(31), Some(P8Num::new(0.0)));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
		self.0.checked_shl(rhs).map(Self)
	}
	
	/// Unbounded shift left. Computes `self << rhs`, without bounding the value of `rhs`.
	///
	/// If `rhs` is larger or equal to the number of bits in `self`,
	/// the entire value is shifted out, and `0` is returned.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(1.0).unbounded_shl(4), P8Num::new(16.0));
	/// assert_eq!(P8Num::new(1.0).unbounded_shl(129), P8Num::new(0.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn unbounded_shl(self, rhs: u32) -> Self {
		Self(self.0.unbounded_shl(rhs))
	}
	
	/// Checked shift right. Computes `self >> rhs`, returning `None` if `rhs` is
	/// larger than or equal to the number of bits in `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(16.0).checked_shr(4), Some(P8Num::new(1.0)));
	/// assert_eq!(P8Num::new(1.0).checked_shr(129), None);
	/// assert_eq!(P8Num::new(1.0).checked_shr(31), Some(P8Num::new(0.0)));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
		self.0.checked_shr(rhs).map(Self)
	}
	
	/// Checked absolute value. Computes `self.abs()`, returning `None` if `self == MIN`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-7.0).checked_abs(), Some(P8Num::new(7.0)));
	/// assert_eq!(P8Num::MIN.checked_abs(), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_abs(self) -> Option<Self> {
		self.0.checked_abs().map(Self)
	}
	
	/// Checked exponentiation. Computes `self.pow(exp)`, returning `None` if
	/// overflow occurred.
	///
	/// # Examples
	///
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(8.0).checked_pow(P8Num::new(2.0)), Some(P8Num::new(64.0)));
	/// assert_eq!(P8Num::MAX.checked_pow(P8Num::new(2.0)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn checked_pow(self, mut _exp: Self) -> Option<Self> {
		unimplemented!()
	}
	
	/// Saturating integer addition. Computes `self + rhs`, saturating at the numeric
	/// bounds instead of overflowing.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).saturating_add(P8Num::new(1.0)), P8Num::new(101.0));
	/// assert_eq!(P8Num::MAX.saturating_add(P8Num::new(100.0)), P8Num::MAX);
	/// assert_eq!(P8Num::MIN.saturating_add(P8Num::new(-1.0)), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn saturating_add(self, rhs: Self) -> Self {
		Self(self.0.saturating_add(rhs.0))
	}
	
	/// Saturating integer subtraction. Computes `self - rhs`, saturating at the
	/// numeric bounds instead of overflowing.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).saturating_sub(P8Num::new(127.0)), P8Num::new(-27.0));
	/// assert_eq!(P8Num::MIN.saturating_sub(P8Num::new(100.0)), P8Num::MIN);
	/// assert_eq!(P8Num::MAX.saturating_sub(P8Num::new(-1.0)), P8Num::MAX);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn saturating_sub(self, rhs: Self) -> Self {
		Self(self.0.saturating_sub(rhs.0))
	}
	
	/// Saturating integer negation. Computes `-self`, returning `MAX` if `self == MIN`
	/// instead of overflowing.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).saturating_neg(), P8Num::new(-100.0));
	/// assert_eq!(P8Num::new(-100.0).saturating_neg(), P8Num::new(100.0));
	/// assert_eq!(P8Num::MIN.saturating_neg(), P8Num::MAX);
	/// assert_eq!(P8Num::MAX.saturating_neg(), P8Num::MIN + P8Num::EPSILON);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn saturating_neg(self) -> Self {
		Self(self.0.saturating_neg())
	}
	
	/// Saturating absolute value. Computes `self.abs()`, returning `MAX` if `self ==
	/// MIN` instead of overflowing.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).saturating_abs(), P8Num::new(100.0));
	/// assert_eq!(P8Num::new(-100.0).saturating_abs(), P8Num::new(100.0));
	/// assert_eq!(P8Num::MIN.saturating_abs(), P8Num::MAX);
	/// assert_eq!((P8Num::MIN + P8Num::EPSILON).saturating_abs(), P8Num::MAX);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn saturating_abs(self) -> Self {
		Self(self.0.saturating_abs())
	}
	
	/// Saturating integer multiplication. Computes `self * rhs`, saturating at the
	/// numeric bounds instead of overflowing.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(10.0).saturating_mul(P8Num::new(12.0)), P8Num::new(120.0));
	/// assert_eq!(P8Num::MAX.saturating_mul(P8Num::new(10.0)), P8Num::MAX);
	/// assert_eq!(P8Num::MIN.saturating_mul(P8Num::new(10.0)), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn saturating_mul(self, rhs: Self) -> Self {
		const MIN: i64 = P8Num::MIN.0 as i64;
		const MAX: i64 = P8Num::MAX.0 as i64 + 1;
		let res = ((self.0 as i64) * (rhs.0 as i64)) >> 16;
		
		match res {
			   ..MIN => P8Num::MIN,
			MIN..MAX => Self(res as i32),
			MAX..    => P8Num::MAX,
		}
	}
	
	/// Saturating integer division. Computes `self / rhs`, saturating at the
	/// numeric bounds instead of overflowing.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).saturating_div(P8Num::new(2.0)), P8Num::new(2.5));
	/// assert_eq!(P8Num::MAX.saturating_div(P8Num::new(-1.0)), P8Num::MIN + P8Num::EPSILON);
	/// assert_eq!(P8Num::MIN.saturating_div(P8Num::new(-1.0)), P8Num::MAX);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn saturating_div(self, rhs: Self) -> Self {
		const MIN: i64 = P8Num::MIN.0 as i64;
		const MAX: i64 = P8Num::MAX.0 as i64 + 1;
		let res = ((self.0 as i64) << 16) / (rhs.0 as i64);
		
		match res {
			   ..MIN => P8Num::MIN,
			MIN..MAX => Self(res as i32),
			MAX..    => P8Num::MAX,
		}
	}
	
	/// Saturating integer exponentiation. Computes `self.pow(exp)`,
	/// saturating at the numeric bounds instead of overflowing.
	///
	/// # Examples
	///
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-4.0).saturating_pow(3), P8Num::new(-64.0));
	/// assert_eq!(P8Num::MIN.saturating_pow(2), P8Num::MAX);
	/// assert_eq!(P8Num::MIN.saturating_pow(3), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn saturating_pow(self, _exp: u32) -> Self {
		unimplemented!()
	}
	
	/// Wrapping (modular) addition. Computes `self + rhs`, wrapping around at the
	/// boundary of the type.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_add(P8Num::new(27.0)), P8Num::new(127.0));
	/// assert_eq!(P8Num::MAX.wrapping_add(P8Num::EPSILON), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn wrapping_add(self, rhs: Self) -> Self {
		Self(self.0.wrapping_add(rhs.0))
	}
	
	/// Wrapping (modular) subtraction. Computes `self - rhs`, wrapping around at the
	/// boundary of the type.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).wrapping_sub(P8Num::new(127.0)), P8Num::new(-127.0));
	/// assert_eq!(P8Num::MIN.wrapping_sub(P8Num::EPSILON), P8Num::MAX);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn wrapping_sub(self, rhs: Self) -> Self {
		Self(self.0.wrapping_sub(rhs.0))
	}
	
	/// Wrapping (modular) multiplication. Computes `self * rhs`, wrapping around at
	/// the boundary of the type.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(10.0).wrapping_mul(P8Num::new(12.0)), P8Num::new(120.0));
	/// assert_eq!(P8Num::from_raw(0x1010_000F).wrapping_mul(P8Num::from_raw(0x0010_0000)), P8Num::from_raw(0x0100_00F0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn wrapping_mul(self, rhs: Self) -> Self {
		self.overflowing_mul(rhs).0
	}
	
	/// Wrapping (modular) division. Computes `self / rhs`, wrapping around at the
	/// boundary of the type.
	///
	/// The only case where such wrapping can occur is when one divides `MIN / -1` on a signed type (where
	/// `MIN` is the negative minimal value for the type); this is equivalent to `-MIN`, a positive value
	/// that is too large to represent in the type. In such a case, this function returns `MIN` itself.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_div(P8Num::new(10.0)), P8Num::new(10.0));
	/// assert_eq!(P8Num::from_raw(0x0100_00F0).wrapping_div(P8Num::from_raw(0x0010_0000)), P8Num::from_raw(0x0010_000F));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn wrapping_div(self, rhs: Self) -> Self {
		Self((((self.0 as i64) << 16) / (rhs.0 as i64)) as i32)
	}
	
	/// Wrapping Euclidean division. Computes `self.div_euclid(rhs)`,
	/// wrapping around at the boundary of the type.
	///
	/// Wrapping will only occur in `MIN / -1` on a signed type (where `MIN` is the negative minimal value
	/// for the type). This is equivalent to `-MIN`, a positive value that is too large to represent in the
	/// type. In this case, this method returns `MIN` itself.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_div_euclid(P8Num::new(10.0)), P8Num::new(10.0));
	/// assert_eq!(P8Num::MIN.wrapping_div_euclid(P8Num::new(-1.0)), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn wrapping_div_euclid(self, rhs: Self) -> Self {
		Self(((self.0 as i64) << 16).div_euclid(rhs.0 as i64) as i32)
	}
	
	/// Wrapping (modular) remainder. Computes `self % rhs`, wrapping around at the
	/// boundary of the type.
	///
	/// Such wrap-around never actually occurs mathematically; implementation artifacts make `x % y`
	/// invalid for `MIN / -1` on a signed type (where `MIN` is the negative minimal value). In such a case,
	/// this function returns `0`.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_rem(P8Num::new(10.0)), P8Num::new(0.0));
	/// assert_eq!(P8Num::MIN.wrapping_rem(-P8Num::EPSILON), P8Num::new(0.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn wrapping_rem(self, rhs: Self) -> Self {
		Self(self.0.wrapping_rem(rhs.0))
	}
	
	/// Wrapping Euclidean remainder. Computes `self.rem_euclid(rhs)`, wrapping around
	/// at the boundary of the type.
	///
	/// Wrapping will only occur in `MIN % -1` on a signed type (where `MIN` is the negative minimal value
	/// for the type). In this case, this method returns 0.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_rem_euclid(P8Num::new(10.0)), P8Num::new(0.0));
	/// assert_eq!(P8Num::MIN.wrapping_rem_euclid(-P8Num::EPSILON), P8Num::new(0.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn wrapping_rem_euclid(self, rhs: Self) -> Self {
		Self(self.0.wrapping_rem_euclid(rhs.0))
	}
	
	/// Wrapping (modular) negation. Computes `-self`, wrapping around at the boundary
	/// of the type.
	///
	/// The only case where such wrapping can occur is when one negates `MIN` on a signed type (where `MIN`
	/// is the negative minimal value for the type); this is a positive value that is too large to represent
	/// in the type. In such a case, this function returns `MIN` itself.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_neg(), P8Num::new(-100.0));
	/// assert_eq!(P8Num::new(-100.0).wrapping_neg(), P8Num::new(100.0));
	/// assert_eq!(P8Num::MIN.wrapping_neg(), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn wrapping_neg(self) -> Self {
		Self(self.0.wrapping_neg())
	}
	
	/// Panic-free bitwise shift-left; yields `self << mask(rhs)`, where `mask` removes
	/// any high-order bits of `rhs` that would cause the shift to exceed the bitwidth of the type.
	///
	/// Note that this is *not* the same as a rotate-left; the RHS of a wrapping shift-left is restricted to
	/// the range of the type, rather than the bits shifted out of the LHS being returned to the other end.
	/// The primitive integer types all implement a [`rotate_left`](Self::rotate_left) function,
	/// which may be what you want instead.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-1.0).wrapping_shl(7), P8Num::new(-128.0));
	/// assert_eq!(P8Num::new(-1.0).wrapping_shl(128), P8Num::new(-1.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn wrapping_shl(self, rhs: u32) -> Self {
		Self(self.0.wrapping_shl(rhs))
	}
	
	/// Panic-free bitwise shift-right; yields `self >> mask(rhs)`, where `mask`
	/// removes any high-order bits of `rhs` that would cause the shift to exceed the bitwidth of the type.
	///
	/// Note that this is *not* the same as a rotate-right; the RHS of a wrapping shift-right is restricted
	/// to the range of the type, rather than the bits shifted out of the LHS being returned to the other
	/// end. The primitive integer types all implement a [`rotate_right`](Self::rotate_right) function,
	/// which may be what you want instead.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-128.0).wrapping_shr(7), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-128.0).wrapping_shr(64), P8Num::new(-128.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn wrapping_shr(self, rhs: u32) -> Self {
		Self(self.0.wrapping_shr(rhs))
	}
	
	/// Wrapping (modular) absolute value. Computes `self.abs()`, wrapping around at
	/// the boundary of the type.
	///
	/// The only case where such wrapping can occur is when one takes the absolute value of the negative
	/// minimal value for the type; this is a positive value that is too large to represent in the type. In
	/// such a case, this function returns `MIN` itself.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(100.0).wrapping_abs(), P8Num::new(100.0));
	/// assert_eq!(P8Num::new(-100.0).wrapping_abs(), P8Num::new(100.0));
	/// assert_eq!(P8Num::MIN.wrapping_abs(), P8Num::MIN);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn wrapping_abs(self) -> Self {
		Self(self.0.wrapping_abs())
	}
	
	/// Calculates `self` + `rhs`.
	///
	/// Returns a tuple of the addition along with a boolean indicating
	/// whether an arithmetic overflow would occur. If an overflow would have
	/// occurred then the wrapped value is returned.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).overflowing_add(P8Num::new(2.0)), (P8Num::new(7.0), false));
	/// assert_eq!(P8Num::MAX.overflowing_add(P8Num::EPSILON), (P8Num::MIN, true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
		let (res, overflow) = self.0.overflowing_add(rhs.0);
		(Self(res), overflow)
	}
	
	/// Calculates `self` - `rhs`.
	///
	/// Returns a tuple of the subtraction along with a boolean indicating whether an arithmetic overflow
	/// would occur. If an overflow would have occurred then the wrapped value is returned.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).overflowing_sub(P8Num::new(2.0)), (P8Num::new(3.0), false));
	/// assert_eq!(P8Num::MIN.overflowing_sub(P8Num::EPSILON), (P8Num::MAX, true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
		let (res, overflow) = self.0.overflowing_sub(rhs.0);
		(Self(res), overflow)
	}
	
	/// Calculates the multiplication of `self` and `rhs`.
	///
	/// Returns a tuple of the multiplication along with a boolean indicating whether an arithmetic overflow
	/// would occur. If an overflow would have occurred then the wrapped value is returned.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).overflowing_mul(P8Num::new(2.0)), (P8Num::new(10.0), false));
	/// assert_eq!(P8Num::new(30000.0).overflowing_mul(P8Num::new(2.0)), (P8Num::new(-5536.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
		const MIN: i64 = P8Num::MIN.0 as i64;
		const MAX: i64 = P8Num::MAX.0 as i64 + 1;
		
		let res = ((self.0 as i64) * (rhs.0 as i64)) >> 16;
		let overflow = match res {
			   ..MIN => true,
			MIN..MAX => false,
			MAX..    => true,
		};
		
		(Self(res as i32), overflow)
	}
	
	/// Calculates the divisor when `self` is divided by `rhs`.
	///
	/// Returns a tuple of the divisor along with a boolean indicating whether an arithmetic overflow would
	/// occur. If an overflow would occur then self is returned.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).overflowing_div(P8Num::new(2.0)), (P8Num::new(2.5), false));
	/// assert_eq!(P8Num::MIN.overflowing_div(P8Num::new(-1.0)), (P8Num::MIN, true));
	/// assert_eq!(P8Num::new(30000.0).overflowing_div(P8Num::new(0.5)), (P8Num::new(-5536.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
		const MIN: i64 = P8Num::MIN.0 as i64;
		const MAX: i64 = P8Num::MAX.0 as i64 + 1;
		
		let res = ((self.0 as i64) << 16) / (rhs.0 as i64);
		let overflow = match res {
			   ..MIN => true,
			MIN..MAX => false,
			MAX..    => true,
		};
		
		(Self(res as i32), overflow)
	}
	
	/// Calculates the quotient of Euclidean division `self.div_euclid(rhs)`.
	///
	/// Returns a tuple of the divisor along with a boolean indicating whether an arithmetic overflow would
	/// occur. If an overflow would occur then `self` is returned.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-5.0).overflowing_div_euclid(P8Num::new(2.0)), (P8Num::new(-3.0), false));
	/// assert_eq!(P8Num::MIN.overflowing_div_euclid(P8Num::new(-1.0)), (P8Num::MIN, true));
	/// assert_eq!(P8Num::new(30000.0).overflowing_div_euclid(P8Num::new(0.5)), (P8Num::new(-5536.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool) {
		const MIN: i64 = P8Num::MIN.0 as i64;
		const MAX: i64 = P8Num::MAX.0 as i64 + 1;
		
		let res = (self.0 as i64).div_euclid(rhs.0 as i64) << 16;
		let overflow = match res {
			   ..MIN => true,
			MIN..MAX => false,
			MAX..    => true,
		};
		
		(Self(res as i32), overflow)
	}
	
	/// Calculates the remainder when `self` is divided by `rhs`.
	///
	/// Returns a tuple of the remainder after dividing along with a boolean indicating whether an
	/// arithmetic overflow would occur. If an overflow would occur then 0 is returned.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).overflowing_rem(P8Num::new(2.0)), (P8Num::new(1.0), false));
	/// assert_eq!(P8Num::MIN.overflowing_rem(-P8Num::EPSILON), (P8Num::new(0.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
		let (res, overflow) = self.0.overflowing_rem(rhs.0);
		(Self(res), overflow)
	}
	
	/// Overflowing Euclidean remainder. Calculates `self.rem_euclid(rhs)`.
	///
	/// Returns a tuple of the remainder after dividing along with a boolean indicating whether an
	/// arithmetic overflow would occur. If an overflow would occur then 0 is returned.
	///
	/// # Panics
	///
	/// This function will panic if `rhs` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(5.0).overflowing_rem_euclid(P8Num::new(2.0)), (P8Num::new(1.0), false));
	/// assert_eq!(P8Num::MIN.overflowing_rem_euclid(-P8Num::EPSILON), (P8Num::new(0.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	#[track_caller]
	pub const fn overflowing_rem_euclid(self, rhs: Self) -> (Self, bool) {
		let (res, overflow) = self.0.overflowing_rem_euclid(rhs.0);
		(Self(res), overflow)
	}
	
	/// Negates self, overflowing if this is equal to the minimum value.
	///
	/// Returns a tuple of the negated version of self along with a boolean indicating whether an overflow
	/// happened. If `self` is the minimum value (e.g., `P8Num::MIN` for values of type `i32`), then the
	/// minimum value will be returned again and `true` will be returned for an overflow happening.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(2.0).overflowing_neg(), (P8Num::new(-2.0), false));
	/// assert_eq!(P8Num::MIN.overflowing_neg(), (P8Num::MIN, true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_neg(self) -> (Self, bool) {
		let (res, overflow) = self.0.overflowing_neg();
		(Self(res), overflow)
	}
	
	/// Shifts self left by `rhs` bits.
	///
	/// Returns a tuple of the shifted version of self along with a boolean indicating whether the shift
	/// value was larger than or equal to the number of bits. If the shift value is too large, then value is
	/// masked (N-1) where N is the number of bits, and this value is then used to perform the shift.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x0001_0000).overflowing_shl(4), (P8Num::from_raw(0x0010_0000), false));
	/// assert_eq!(P8Num::from_raw(0x0001_0000).overflowing_shl(36), (P8Num::from_raw(0x0010_0000), true));
	/// assert_eq!(P8Num::from_raw(0x0010_0000).overflowing_shl(31), (P8Num::new(0.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_shl(self, rhs: u32) -> (Self, bool) {
		(self.wrapping_shl(rhs), rhs >= 16)
	}
	
	/// Shifts self right by `rhs` bits.
	///
	/// Returns a tuple of the shifted version of self along with a boolean indicating whether the shift
	/// value was larger than or equal to the number of bits. If the shift value is too large, then value is
	/// masked (N-1) where N is the number of bits, and this value is then used to perform the shift.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_raw(0x0010_0000).overflowing_shr(4), (P8Num::from_raw(0x0001_0000), false));
	/// assert_eq!(P8Num::from_raw(0x0010_0000).overflowing_shr(36), (P8Num::from_raw(0x0001_0000), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_shr(self, rhs: u32) -> (Self, bool) {
		(self.wrapping_shr(rhs), rhs >= 16)
	}
	
	/// Computes the absolute value of `self`.
	///
	/// Returns a tuple of the absolute version of self along with a boolean indicating whether an overflow
	/// happened. If self is the minimum value
	/// (e.g., P8Num::MIN for values of type i32),
	/// then the minimum value will be returned again and true will be returned
	/// for an overflow happening.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(10.0).overflowing_abs(), (P8Num::new(10.0), false));
	/// assert_eq!(P8Num::new(-10.0).overflowing_abs(), (P8Num::new(10.0), false));
	/// assert_eq!(P8Num::MIN.overflowing_abs(), (P8Num::MIN, true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_abs(self) -> (Self, bool) {
		(self.wrapping_abs(), self.0 == i32::MIN)
	}
	
	/// Raises self to the power of `exp`, using exponentiation by squaring.
	///
	/// Returns a tuple of the exponentiation along with a bool indicating
	/// whether an overflow happened.
	///
	/// # Examples
	///
	/// ```should_panic
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(3.0).overflowing_pow(P8Num::new(4.0)), (P8Num::new(81.0), false));
	/// assert_eq!(P8Num::new(10.0).overflowing_pow(P8Num::new(4.0)), (P8Num::new(32768.0), true));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn overflowing_pow(self, mut _exp: Self) -> (Self, bool) {
		unimplemented!()
	}
	
	/// Returns the memory representation of this integer as a byte array in
	/// big-endian (network) byte order.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// let bytes = P8Num::from_raw(0x1234_5678).to_be_bytes();
	/// assert_eq!(bytes, [0x12, 0x34, 0x56, 0x78]);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn to_be_bytes(self) -> [u8; 4] {
		self.0.to_be_bytes()
	}
	
	/// Returns the memory representation of this integer as a byte array in
	/// little-endian byte order.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// let bytes = P8Num::from_raw(0x1234_5678).to_le_bytes();
	/// assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn to_le_bytes(self) -> [u8; 4] {
		self.0.to_le_bytes()
	}
	
	/// Returns the memory representation of this integer as a byte array in
	/// native byte order.
	///
	/// As the target platform's native endianness is used, portable code
	/// should use [`to_be_bytes`] or [`to_le_bytes`], as appropriate,
	/// instead.
	///
	/// [`to_be_bytes`]: Self::to_be_bytes
	/// [`to_le_bytes`]: Self::to_le_bytes
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// let bytes = P8Num::from_raw(0x1234_5678).to_ne_bytes();
	/// assert_eq!(
	///     bytes,
	///     if cfg!(target_endian = "big") {
	///         [0x12, 0x34, 0x56, 0x78]
	///     } else {
	///         [0x78, 0x56, 0x34, 0x12]
	///     }
	/// );
	/// ```
	#[allow(unnecessary_transmutes)]
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn to_ne_bytes(self) -> [u8; 4] {
		self.0.to_ne_bytes()
	}
	
	/// Creates an integer value from its representation as a byte array in
	/// big endian.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// let value = P8Num::from_be_bytes([0x12, 0x34, 0x56, 0x78]);
	/// assert_eq!(value, P8Num::from_raw(0x1234_5678));
	/// ```
	///
	/// When starting from a slice rather than an array, fallible conversion APIs can be used:
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// fn read_be_i32(input: &mut &[u8]) -> P8Num {
	///     let (int_bytes, rest) = input.split_at(size_of::<P8Num>());
	///     *input = rest;
	///     P8Num::from_be_bytes(int_bytes.try_into().unwrap())
	/// }
	/// ```
	#[must_use]
	#[inline]
	pub const fn from_be_bytes(bytes: [u8; 4]) -> Self {
		Self(i32::from_be_bytes(bytes))
	}
	
	/// Creates an integer value from its representation as a byte array in
	/// little endian.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// let value = P8Num::from_le_bytes([0x78, 0x56, 0x34, 0x12]);
	/// assert_eq!(value, P8Num::from_raw(0x1234_5678));
	/// ```
	///
	/// When starting from a slice rather than an array, fallible conversion APIs can be used:
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// fn read_le_i32(input: &mut &[u8]) -> P8Num {
	///     let (int_bytes, rest) = input.split_at(size_of::<P8Num>());
	///     *input = rest;
	///     P8Num::from_le_bytes(int_bytes.try_into().unwrap())
	/// }
	/// ```
	#[must_use]
	#[inline]
	pub const fn from_le_bytes(bytes: [u8; 4]) -> Self {
		Self(i32::from_le_bytes(bytes))
	}
	
	/// Creates an integer value from its memory representation as a byte
	/// array in native endianness.
	///
	/// As the target platform's native endianness is used, portable code
	/// likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as
	/// appropriate instead.
	///
	/// [`from_be_bytes`]: Self::from_be_bytes
	/// [`from_le_bytes`]: Self::from_le_bytes
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// let value = P8Num::from_ne_bytes(if cfg!(target_endian = "big") {
	///     [0x12, 0x34, 0x56, 0x78]
	/// } else {
	///     [0x78, 0x56, 0x34, 0x12]
	/// });
	/// assert_eq!(value, P8Num::from_raw(0x1234_5678));
	/// ```
	///
	/// When starting from a slice rather than an array, fallible conversion APIs can be used:
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// fn read_ne_i32(input: &mut &[u8]) -> P8Num {
	///     let (int_bytes, rest) = input.split_at(size_of::<P8Num>());
	///     *input = rest;
	///     P8Num::from_ne_bytes(int_bytes.try_into().unwrap())
	/// }
	/// ```
	#[allow(unnecessary_transmutes)]
	#[must_use]
	#[inline]
	pub const fn from_ne_bytes(bytes: [u8; 4]) -> Self {
		Self(i32::from_ne_bytes(bytes))
	}
}

impl const Add for P8Num {
	type Output = P8Num;
	
	fn add(self, rhs: Self) -> P8Num {
		self.saturating_add(rhs)
	}
}

impl AddAssign for P8Num {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl const Sub for P8Num {
	type Output = P8Num;
	
	fn sub(self, rhs: Self) -> P8Num {
		self.saturating_sub(rhs)
	}
}

impl SubAssign for P8Num {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl const Mul for P8Num {
	type Output = P8Num;
	
	fn mul(self, rhs: Self) -> P8Num {
		self.saturating_mul(rhs)
	}
}

impl MulAssign for P8Num {
	fn mul_assign(&mut self, rhs: Self) {
		*self = *self * rhs;
	}
}

impl const Div for P8Num {
	type Output = P8Num;
	
	fn div(self, rhs: Self) -> P8Num {
		self.saturating_div(rhs)
	}
}

impl DivAssign for P8Num {
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

impl RemAssign for P8Num {
	fn rem_assign(&mut self, rhs: Self) {
		*self = *self / rhs;
	}
}

impl BitAnd for P8Num {
	type Output = P8Num;
	
	fn bitand(self, rhs: Self) -> P8Num {
		Self(self.0 & rhs.0)
	}
}

impl BitAndAssign for P8Num {
	fn bitand_assign(&mut self, rhs: Self) {
		*self = *self & rhs;
	}
}

impl BitOr for P8Num {
	type Output = P8Num;
	
	fn bitor(self, rhs: Self) -> P8Num {
		Self(self.0 | rhs.0)
	}
}

impl BitOrAssign for P8Num {
	fn bitor_assign(&mut self, rhs: Self) {
		*self = *self | rhs;
	}
}

impl BitXor for P8Num {
	type Output = P8Num;
	
	fn bitxor(self, rhs: Self) -> P8Num {
		Self(self.0 ^ rhs.0)
	}
}

impl BitXorAssign for P8Num {
	fn bitxor_assign(&mut self, rhs: Self) {
		*self = *self ^ rhs;
	}
}

impl Not for P8Num {
	type Output = P8Num;
	
	fn not(self) -> P8Num {
		Self(!self.0)
	}
}

impl Neg for P8Num {
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
		write!(f, "P8Num(0x{:04X}.{:04X} = {})", self.integer(), self.fractional(), f64::from(self))
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

impl const From<&P8Num> for f32 {
	fn from(value: &P8Num) -> f32 {
		f32::from(*value)
	}
}

impl const From<&P8Num> for f64 {
	fn from(value: &P8Num) -> f64 {
		f64::from(*value)
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

//TODO: use feature(const_result_trait_fn)
const fn try_into_some(value: i64) -> Option<i32> {
	match value.try_into() {
		Ok(ok) => Some(ok),
		Err(_) => None,
	}
}
