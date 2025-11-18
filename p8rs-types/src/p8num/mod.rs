//! Pico-8 fixed point math.

use core::fmt::{Debug, Display, Formatter};
use core::num::FpCategory;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use arrayvec::{ArrayVec, ArrayString};
use bitflags::bitflags;

pub mod consts;
mod from_ascii;

/// 16.16-bit fixed point number type.
///
/// [P8Num] uses 16 bits for integer part and 16 bits for fractional part. It can represent values from -32768.0 to 32767.9999847412109375 inclusive.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "gc-arena", derive(gc_arena::Collect), collect(require_static))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct P8Num(i32);

impl P8Num {
	/// The smallest value that can be represented by [P8Num] (-32768.0 or 0x8000.0000).
	pub const MIN: Self = Self(i32::MIN);
	
	/// The largest value that can be represented by [P8Num] (32767.9999847412109375 or 0x7FFF.FFFF).
	pub const MAX: Self = Self(i32::MAX);
	
	/// The additive identity of [P8Num], commonly called zero (0.0 or 0x0000.0000).
	pub const ZERO: Self = Self(0);
	
	/// The multiplicative identity of [P8Num], commonly called one (1.0 or 0x0001.0000).
	pub const ONE: Self = Self(1 << 16);
	
	/// The smallest positive value of [P8Num] (0.00001525878 or 0x0000.0001).
	pub const EPSILON: Self = Self(1);
	
	/// The fractional part bits mask (0x0000.FFFF).
	pub const FRACT_BITS: Self = Self(0x0000_FFFF);
	
	/// The integer part bits mask (0xFFFF.0000).
	pub const INTEGER_BITS: Self = Self(0xFFFF_0000_u32 as i32);
	
	/// Euler's number (e).
	pub const E: Self = Self(0x0002_b7e2);
	
	/// Creates new value from `f32`.
	/// 
	/// Overflow is handled in a wrapping manner. Infinities become MIN/MAX and NaN becomes 0.
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
	/// assert_eq!(P8Num::new(32768.0), P8Num::new(-32768.0));
	/// assert_eq!(P8Num::new(-32769.0), P8Num::new(32767.0));
	/// assert_eq!(P8Num::new(f32::INFINITY), P8Num::MAX);
	/// assert_eq!(P8Num::new(f32::NEG_INFINITY), P8Num::MIN);
	/// assert_eq!(P8Num::new(f32::NAN), P8Num::ZERO);
	/// ```
	pub const fn new(value: f32) -> Self {
		match value.classify() {
			FpCategory::Zero |
			FpCategory::Subnormal |
			FpCategory::Nan => P8Num::ZERO,
			FpCategory::Infinite => if value.is_sign_positive() { P8Num::MAX } else { P8Num::MIN },
			FpCategory::Normal => {
				let bits: u32 = value.to_bits();
				let negative = bits >> 31 != 0;
				let exponent = ((bits >> 23) & 0xff) as i32 - 127 - 23 + 16;
				let mantissa = (bits & 0x7fffff) | 0x800000;
				
				let mut units = match exponent {
					..0 => mantissa.unbounded_shr((-exponent) as u32),
					0.. => mantissa.unbounded_shl(exponent as u32),
				} as i32;
				
				if negative {
					units = (!units).wrapping_add(1)
				}
				
				Self(units)
			}
		}
	}
	
	/// Creates new value from `f64`.
	///
	/// Overflow is handled in a wrapping manner. Infinities become MIN/MAX and NaN becomes 0.
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
	/// assert_eq!(P8Num::new_f64(32768.0), P8Num::new(-32768.0));
	/// assert_eq!(P8Num::new_f64(-32769.0), P8Num::new(32767.0));
	/// assert_eq!(P8Num::new_f64(f64::INFINITY), P8Num::MAX);
	/// assert_eq!(P8Num::new_f64(f64::NEG_INFINITY), P8Num::MIN);
	/// assert_eq!(P8Num::new_f64(f64::NAN), P8Num::ZERO);
	/// ```
	pub const fn new_f64(value: f64) -> Self {
		match value.classify() {
			FpCategory::Zero |
			FpCategory::Subnormal |
			FpCategory::Nan => P8Num::ZERO,
			FpCategory::Infinite => if value.is_sign_positive() { P8Num::MAX } else { P8Num::MIN },
			FpCategory::Normal => {
				let bits: u64 = value.to_bits();
				let negative = bits >> 63 != 0;
				let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023 - 52 + 16;
				let mantissa = (bits & 0xfffffffffffff) | 0x10000000000000;
				
				let mut units = match exponent {
					..0 => mantissa.unbounded_shr((-exponent) as u32),
					0.. => mantissa.unbounded_shl(exponent as u32),
				} as i32;
				
				if negative {
					units = (!units).wrapping_add(1)
				}
				
				Self(units)
			}
		}
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
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_ascii(b"+10"), Ok(P8Num::new(10.0)));
	/// ```
	/// Trailing space returns error:
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!(P8Num::from_ascii(b"1 ").is_err());
	/// ```
	#[inline]
	pub fn from_ascii(src: &[u8]) -> Result<Self, FromAsciiError> {
		Self::from_ascii_radix(src, 10)
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
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::from_ascii_radix(b"A", 16), Ok(P8Num::new(10.0)));
	/// ```
	/// Trailing space returns error:
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!(P8Num::from_ascii_radix(b"1 ", 10).is_err());
	/// ```
	#[inline]
	pub fn from_ascii_radix(src: &[u8], radix: u32) -> Result<Self, FromAsciiError> {
		match radix {
			2 => from_ascii::from_ascii_bin(src),
			10 => from_ascii::from_ascii_dec(src),
			16 => from_ascii::from_ascii_hex(src),
			_ => unreachable!(),
		}
	}
	
	/// Formats `self` into an ASCII-byte slice.
	/// 
	/// See also [Self::to_str].
	///
	/// # Examples
	///
	/// ```
	/// #![feature(ascii_char)]
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).to_ascii().as_ref().as_str(), "0");
	/// assert_eq!(P8Num::new(0.5).to_ascii().as_ref().as_str(), "0.5");
	/// assert_eq!(P8Num::new(-0.5).to_ascii().as_ref().as_str(), "-0.5");
	/// assert_eq!(P8Num::new(0.12345).to_ascii().as_ref().as_str(), "0.1234");
	/// assert_eq!(P8Num::EPSILON.to_ascii().as_ref().as_str(), "0");
	/// assert_eq!((-P8Num::EPSILON).to_ascii().as_ref().as_str(), "-0");
	/// assert_eq!(P8Num::MAX.to_ascii().as_ref().as_str(), "32768");
	/// ```
	pub fn to_ascii(&self) -> impl AsRef<[core::ascii::Char]> {
		self.to_ascii_fmt(P8NumStringConversionFlags::empty())
	}
	
	/// Formats `self` into an ASCII-byte slice.
	///
	/// See also [Self::to_ascii] and [P8NumStringConversionFlags].
	///
	/// # Examples
	///
	/// ```
	/// #![feature(ascii_char)]
	/// use p8rs_types::p8num::{P8Num, P8NumStringConversionFlags};
	///
	/// assert_eq!(P8Num::new(0.0).to_ascii_fmt(P8NumStringConversionFlags::HEX).as_ref().as_str(), "0x0000.0000");
	/// assert_eq!(P8Num::new(0.125).to_ascii_fmt(P8NumStringConversionFlags::I32).as_ref().as_str(), "8192");
	/// assert_eq!(P8Num::new(-0.5).to_ascii_fmt(P8NumStringConversionFlags::HEX).as_ref().as_str(), "0xffff.8000");
	/// assert_eq!(P8Num::new(1.0).to_ascii_fmt(P8NumStringConversionFlags::HEX.union(P8NumStringConversionFlags::I32)).as_ref().as_str(), "0x00010000");
	/// assert_eq!(P8Num::EPSILON.to_ascii_fmt(P8NumStringConversionFlags::HEX).as_ref().as_str(), "0x0000.0001");
	/// assert_eq!((-P8Num::EPSILON).to_ascii_fmt(P8NumStringConversionFlags::HEX).as_ref().as_str(), "0xffff.ffff");
	/// assert_eq!(P8Num::MAX.to_ascii_fmt(P8NumStringConversionFlags::HEX).as_ref().as_str(), "0x7fff.ffff");
	/// ```
	pub fn to_ascii_fmt(&self, format_flags: P8NumStringConversionFlags) -> impl AsRef<[core::ascii::Char]> {
		use core::ascii::Char;
		use core::fmt::Write;
		
		let is_hex = format_flags.contains(P8NumStringConversionFlags::HEX);
		let is_i32 = format_flags.contains(P8NumStringConversionFlags::I32);
		
		let mut value = *self;
		if !is_hex && !is_i32 && !(P8Num::from_raw(0x0000_0007) ..= P8Num::from_raw(0x0000_FFF9)).contains(&value.fract()) {
			value = value.round();
		}
		
		let mut string = ArrayString::<16>::new_const();
		if self.is_negative() && !is_hex {
			write!(&mut string, "-").unwrap();
		}
		if is_i32 {
			if is_hex {
				write!(&mut string, "0x{:08x}", value.to_raw()).unwrap();
			} else {
				write!(&mut string, "{}", value.to_raw()).unwrap();
			}
		} else {
			if is_hex {
				// write!(&mut string, "0x{:04x}.{:04x}", value.to_integer(), value.to_raw() & 0xFFFF).unwrap();
				write!(&mut string, "0x{:04x}", value.to_integer()).unwrap();
				write!(&mut string, ".{:04x}", value.to_raw() & 0xFFFF).unwrap();
			} else {
				write!(&mut string, "{:.4}", f64::from(value).abs()).unwrap();
			}
		}
		
		let mut buffer = ArrayVec::<_, 16>::new_const();
		buffer.extend(string.as_ascii().unwrap().iter().copied());
		
		if !is_hex && !is_i32 {
			while let Some(Char::Digit0) = buffer.last() {
				buffer.pop();
			}
			
			if let Some(Char::FullStop) = buffer.last() {
				buffer.pop();
			}
		}
		
		buffer
	}
	
	/// Formats `self` into a &str.
	/// 
	/// See [Self::to_ascii] for more info.
	pub fn to_str(&self) -> impl AsRef<str> {
		self.to_str_fmt(P8NumStringConversionFlags::empty())
	}
	
	/// Formats `self` into a &str with specified format flags.
	///
	/// See [Self::to_ascii_fmt] and [P8NumStringConversionFlags] for more info.
	pub fn to_str_fmt(&self, format_flags: P8NumStringConversionFlags) -> impl AsRef<str> {
		use core::ascii::Char;
		
		struct CharToStr<T: AsRef<[Char]>>(T);
		impl<T: AsRef<[Char]>> AsRef<str> for CharToStr<T> {
			fn as_ref(&self) -> &str {
				self.0.as_ref().as_str()
			}
		}
		
		CharToStr(self.to_ascii_fmt(format_flags))
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
	/// assert_eq!(P8Num::from_raw(-0x0001_0000), P8Num::new(-1.0));
	/// assert_eq!(P8Num::from_raw(-0x0000_8000), P8Num::new(-0.5));
	/// assert_eq!(P8Num::from_raw(0x0000_8000), P8Num::new(0.5));
	/// assert_eq!(P8Num::from_raw(0x0001_0000), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(0x1234_5678), P8Num::new_f64(4660.3377685546875));
	/// ```
	#[inline]
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
	/// assert_eq!(P8Num::new(-1.0).to_raw(), -0x0001_0000);
	/// assert_eq!(P8Num::new(-0.5).to_raw(), -0x0000_8000);
	/// assert_eq!(P8Num::new(0.5).to_raw(), 0x0000_8000);
	/// assert_eq!(P8Num::new(1.0).to_raw(), 0x0001_0000);
	/// assert_eq!(P8Num::new_f64(4660.3377685546875).to_raw(), 0x1234_5678);
	/// ```
	#[inline]
	pub const fn to_raw(self) -> i32 {
		self.0
	}
	
	pub const fn to_integer(self) -> i16 {
		(self.0 >> 16) as i16
	}
	
	/// Returns the integer part of `self`.
	/// 
	/// This means that non-integer numbers are always truncated towards zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-1.5).trunc(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-0.5).trunc(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(0.5).trunc(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(1.5).trunc(), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).trunc(), P8Num::new(0x1234 as f32));
	/// ```
	#[must_use = "method returns a new number and does not mutate the original value"]
	#[inline]
	pub const fn trunc(self) -> Self {
		if self.is_negative() {
			self.ceil()
		} else {
			self.floor()
		}
	}
	
	/// Returns the fractional part of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-1.0).fract(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(-0.5).fract(), P8Num::new(0.5));
	/// assert_eq!(P8Num::new(-0.2).fract(), P8Num::new(0.2));
	/// assert_eq!(P8Num::new(0.2).fract(), P8Num::new(0.2));
	/// assert_eq!(P8Num::new(0.5).fract(), P8Num::new(0.5));
	/// assert_eq!(P8Num::new(1.0).fract(), P8Num::new(0.0));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).fract(), P8Num::from_raw(0x0000_5678));
	/// ```
	#[must_use = "method returns a new number and does not mutate the original value"]
	#[inline]
	pub const fn fract(self) -> Self {
		(self - self.trunc()).abs()
	}
	
	/// Returns the largest integer less than or equal to `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-1.0).floor(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-0.5).floor(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-0.2).floor(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(0.2).floor(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(0.5).floor(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(1.0).floor(), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).floor(), P8Num::new(0x1234 as f32));
	/// ```
	#[must_use = "method returns a new number and does not mutate the original value"]
	#[inline]
	pub const fn floor(self) -> Self {
		Self((self.0 as u32 & 0xFFFF_0000) as i32)
	}
	
	/// Returns the smallest integer greater than or equal to `self`.
	/// 
	/// Numbers larger than 32767.0 get wrapped around to -32767.0
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-1.0).ceil(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-0.5).ceil(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(-0.2).ceil(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(0.2).ceil(), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(0.5).ceil(), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(1.0).ceil(), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).ceil(), P8Num::new(0x1235 as f32));
	/// ```
	#[must_use = "method returns a new number and does not mutate the original value"]
	#[inline]
	pub const fn ceil(self) -> Self {
		Self(self.0.wrapping_add(0xFFFF)).floor()
	}
	
	/// Returns the nearest integer to `self`. If a value is half-way between two
	/// integers, round away from `0.0`.
	/// 
	/// Numbers larger or equal to 32767.5 gets wrapped around to -32767.0
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(-1.0).round(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-0.5).round(), P8Num::new(-1.0));
	/// assert_eq!(P8Num::new(-0.2).round(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(0.2).round(), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(0.5).round(), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(1.0).round(), P8Num::new(1.0));
	/// assert_eq!(P8Num::from_raw(0x1234_5678).round(), P8Num::new(0x1234 as f32));
	/// ```
	#[must_use = "method returns a new number and does not mutate the original value"]
	#[inline]
	pub const fn round(self) -> Self {
		let midpoint = if self.is_negative() { 0x7FFF } else { 0x8000 };
		Self(self.0.wrapping_add(midpoint)).floor()
	}
	
	/// Computes the absolute value of `self`.
	///
	/// # Overflow behavior
	///
	/// The absolute value of `P8Num::MIN` cannot be represented as an `i32`, and attempting
	/// to calculate it will cause an overflow. This means that code in debug mode will trigger
	/// a panic on this case and optimized code will return `P8Num::MIN` without a panic. If you
	/// do not want this behavior, consider using [`checked_abs`](Self::checked_abs) instead.
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
	
	/// Takes the reciprocal (inverse) of a number, `1/x`.
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::new(0.25).recip(), P8Num::new(4.0));
	/// assert_eq!(P8Num::new(10.0).recip(), P8Num::new(0.1));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub const fn recip(self) -> Self {
		P8Num::ONE / self
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
	
	/// Returns `e^(self)`, (the exponential function).
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!((P8Num::new( 0.0).exp() - P8Num::new(1.0)    ).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::new( 1.0).exp() - P8Num::E           ).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::new( 2.0).exp() - P8Num::E * P8Num::E).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::new( 2.5).exp() - P8Num::new(12.1825)).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::new(-1.0).exp() - P8Num::E.recip()   ).abs() <= P8Num::new(0.001));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub fn exp(self) -> Self {
		let mut fract = self; 
		fract &= P8Num::FRACT_BITS; // Fold to [0.0, 1.0)
		fract <<= 1;                // Expand to [0.0, 2.0)
		fract -= P8Num::ONE;        // Shift to [-1.0, 1.0)
		
		// exp(x): 0 <= x <= 1
		//   ≈ 1.6487274169921875 + x *  0.8242254602827116 + x^2 *  0.206085205078125 + x^3 +  0.034881591796875 + x^4 * 0.00433349609375
		//   = 1.6487274169921875 + x * (0.8242254602827116 +  x  * (0.206085205078125 +  x  + (0.034881591796875 +  x  * 0.00433349609375)))
		let res = P8Num::from_raw(0x0001_A613)
			+ fract * (P8Num::from_raw(0x0000_D300)
				+ fract * (P8Num::from_raw(0x0000_34C2)
					+ fract * (P8Num::from_raw(0x0000_08EE)
						+ fract * P8Num::from_raw(0x0000_011C))));
		
		res * P8Num::E.powi(i32::from(self.floor()))
	}
	
	/// Returns the natural logarithm of the number.
	///
	/// This returns `None` when the number is less or equal zero.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert!((P8Num::new(1.0).ln().unwrap()       - P8Num::new( 0.0)).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::E.ln().unwrap()              - P8Num::new( 1.0)).abs() <= P8Num::new(0.001));
	/// assert!(((P8Num::E * P8Num::E).ln().unwrap() - P8Num::new( 2.0)).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::new(12.1825).ln().unwrap()   - P8Num::new( 2.5)).abs() <= P8Num::new(0.001));
	/// assert!((P8Num::E.recip().ln().unwrap()      - P8Num::new(-1.0)).abs() <= P8Num::new(0.001));
	/// assert_eq!(P8Num::ZERO.ln(), None);
	/// assert_eq!((-P8Num::ONE).ln(), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub fn ln(self) -> Option<Self> {
		if self <= P8Num::ZERO {
			return None;
		}
		
		let mut fract = self;
		let s = 14 - fract.leading_zeros() as i16; // Calculate floor(log2(self))
		if s < 0 { fract <<= (-s) as u32 }         // Wrap to [2.0, 4.0) between each powers of 2
		else     { fract >>=   s  as u32 }
		fract -= P8Num::new(3.0);                  // Shift to [-1.0, 1.0)
		
		// exp(x): 0 <= x <= 1
		//   ≈ 0.4054718017578125 + x *  0.3330535888671875 + x^2 *  -0.05548095703125 + x^3 +  0.013458251953125 + x^4 * -0.0034027099609375
		//   = 0.4054718017578125 + x * (0.3330535888671875 +  x  * (-0.05548095703125 +  x  + (0.013458251953125 +  x  * -0.0034027099609375)))
		let res = P8Num::from_raw(0x0000_67CD)
			+ fract * (P8Num::from_raw(0x0000_5543)
				+ fract * (P8Num::from_raw(-0x0000_0E34)
					+ fract * (P8Num::from_raw(0x0000_0372)
						+ fract * P8Num::from_raw(-0x0000_00DF))));
		
		Some(res + P8Num::from_raw(0x000_b172) * P8Num::from(s + 1)) // log(a) = log(a/n) + log(2)*n
	}
	
	/// Raises a number to an integer power.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(3.0).powi(4), P8Num::new(81.0));
	/// assert_eq!(P8Num::new(0.5).powi(3), P8Num::new(0.125));
	/// assert_eq!(P8Num::new(0.5).powi(-3), P8Num::new(8.0));
	/// assert_eq!(P8Num::new(0.0).powi(2), P8Num::new(0.0));
	/// assert_eq!(P8Num::new(3.0).powi(0), P8Num::new(1.0));
	/// assert_eq!(P8Num::new(0.0).powi(0), P8Num::new(1.0));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub const fn powi(self, exp: i32) -> Self {
		if exp < 0 {
			self.recip().powi(-exp)
		} else if exp == 0 {
			P8Num::ONE
		} else if exp % 2 == 0 {
			(self * self).powi(exp / 2)
		} else {
			self * (self * self).powi(exp / 2)
		}
	}
	
	/// Raises a number to a real power.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert!((P8Num::new( 3.0).powf(P8Num::new( 4.0)).unwrap() - P8Num::new(81.0  )).abs() < P8Num::new(0.001));
	/// assert!((P8Num::new( 3.0).powf(P8Num::new( 0.0)).unwrap() - P8Num::new( 1.0  )).abs() < P8Num::new(0.001));
	/// assert!((P8Num::new( 0.5).powf(P8Num::new( 3.0)).unwrap() - P8Num::new( 0.125)).abs() < P8Num::new(0.001));
	/// assert!((P8Num::new( 0.5).powf(P8Num::new(-3.0)).unwrap() - P8Num::new( 8.0  )).abs() < P8Num::new(0.001));
	/// assert!((P8Num::new(81.0).powf(P8Num::new(0.25)).unwrap() - P8Num::new( 3.0  )).abs() < P8Num::new(0.001));
	/// assert_eq!(P8Num::new( 0.0).powf(P8Num::new(2.0)), Some(P8Num::new(0.0)));
	/// assert_eq!(P8Num::new( 3.0).powf(P8Num::new(0.0)), Some(P8Num::new(1.0)));
	/// assert_eq!(P8Num::new( 0.0).powf(P8Num::new(0.0)), Some(P8Num::new(1.0)));
	/// assert_eq!(P8Num::new(-1.0).powf(P8Num::new(0.5)), None);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub fn powf(self, exp: Self) -> Option<Self> {
		if exp == P8Num::ZERO {
			return Some(P8Num::ONE);
		} else if self == P8Num::ZERO {
			return Some(P8Num::ZERO);
		}
		
		let e_int = i32::from(exp.floor());
		let e_frc = exp & P8Num::FRACT_BITS;
		
		let mut res = self.powi(e_int); // x^y = x^floor(y) * x^fract(y)
		if e_frc != P8Num::ZERO {       //     = x^floor(y) * e^(fract(y) * log(x))
			if self < P8Num::ZERO {
				return None; // Special case
			}
			res *= P8Num::exp(e_frc * self.ln().unwrap());
		}
		
		Some(res)
	}
	
	/// Computes the sine of a number (in turns).
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert!((P8Num::new(0.000).sin() - P8Num::new( 0.0000)).abs() == P8Num::ZERO);
	/// assert!((P8Num::new(0.125).sin() - P8Num::new(-0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.250).sin() - P8Num::new(-1.0000)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.375).sin() - P8Num::new(-0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.500).sin() - P8Num::new( 0.0000)).abs() == P8Num::ZERO);
	/// assert!((P8Num::new(0.625).sin() - P8Num::new( 0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.750).sin() - P8Num::new( 1.0000)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.875).sin() - P8Num::new( 0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(1.000).sin() - P8Num::new( 0.0000)).abs() == P8Num::ZERO);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub fn sin(self) -> Self {
		let mut x = self + P8Num::from_raw(2);    // Nudge forward for later rounding to emulate pico8 precision
		x <<= 2;                                  // Expand angle domain to [0.0, 4.0)
		let flip = x.to_raw() & 0x0002_0000 == 0; // [2.0, 4.0) is just [0.0, 2.0) * -1
		x &= P8Num::from_raw(0x0001_FFF0);        // Fold to [0.0, 2.0) and round up closest 4
		x -= P8Num::ONE;                          // Shift to [-1.0, 1.0)
		
		// cos(xπ/2)
		//   ≈ 1 + x^2 *  -1.233521 + x^4 *  0.252594 + x^6 * 0.019073
		//   = 1 + x^2 * (-1.233521 + x^2 * (0.252594 + x^2 * 0.019073))
		let x2 = x * x;
		let res = P8Num::ONE
			+ x2 * (P8Num::from_raw(-0x0001_3bc8)
				+ x2 * (P8Num::from_raw(0x0000_40aa)
					+ x2 * P8Num::from_raw(-0x0000_04e2)));
		
		if flip { -res } else { res }
	}
	
	/// Computes the cosine of a number (in turns).
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert!((P8Num::new(0.000).cos() - P8Num::new( 1.0000)).abs() == P8Num::ZERO);
	/// assert!((P8Num::new(0.125).cos() - P8Num::new( 0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.250).cos() - P8Num::new( 0.0000)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.375).cos() - P8Num::new(-0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.500).cos() - P8Num::new(-1.0000)).abs() == P8Num::ZERO);
	/// assert!((P8Num::new(0.625).cos() - P8Num::new(-0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.750).cos() - P8Num::new( 0.0000)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(0.875).cos() - P8Num::new( 0.7071)).abs() <= P8Num::EPSILON);
	/// assert!((P8Num::new(1.000).cos() - P8Num::new( 1.0000)).abs() == P8Num::ZERO);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub fn cos(self) -> Self {
		(self + P8Num::new(0.75)).sin()
	}
	
	/// Computes the four quadrant arctangent of `x` and `y` (in turns).
	///
	/// * `x = 0 ∧ y = 0`: -> `0.25`
	/// * `x > 0 ∧ y ≤ 0`: -> `[0, 0.25)`
	/// * `x ≤ 0 ∧ y < 0`: -> `[0.25, 0.5)`
	/// * `x < 0 ∧ y ≥ 0`: -> `[0.5, 0.75)`
	/// * `x ≥ 0 ∧ y > 0`: -> `[0.75, 1)`
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::atan2(P8Num::new(1.0),  P8Num::new(0.0)),  P8Num::new(0.0));
	/// assert_eq!(P8Num::atan2(P8Num::new(1.0),  P8Num::new(1.0)),  P8Num::new(0.875));
	/// assert_eq!(P8Num::atan2(P8Num::new(0.0),  P8Num::new(1.0)),  P8Num::new(0.75));
	/// assert_eq!(P8Num::atan2(P8Num::new(-1.0), P8Num::new(1.0)),  P8Num::new(0.625));
	/// assert_eq!(P8Num::atan2(P8Num::new(-1.0), P8Num::new(0.0)),  P8Num::new(0.5));
	/// assert_eq!(P8Num::atan2(P8Num::new(-1.0), P8Num::new(-1.0)), P8Num::new(0.375));
	/// assert_eq!(P8Num::atan2(P8Num::new(0.0),  P8Num::new(-1.0)), P8Num::new(0.25));
	/// assert_eq!(P8Num::atan2(P8Num::new(1.0),  P8Num::new(-1.0)), P8Num::new(0.125));
	/// assert_eq!(P8Num::atan2(P8Num::new(99.0), P8Num::new(99.0)), P8Num::new(0.875));
	/// assert_eq!(P8Num::atan2(P8Num::new(0.0),  P8Num::new(0.0)),  P8Num::new(0.25));
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	pub fn atan2(x: Self, y: Self) -> Self {
		if x == P8Num::ZERO && y == P8Num::ZERO {
			return P8Num::new(0.25);
		}
		
		let xa = x.abs();
		let ya = y.abs();
		let r = xa.min(ya) / xa.max(ya);
		
		// atan(r)
		//   ≈ r *  0.15899 + r^3 *  -0.05092 + r^5 *  0.02286 + r^7 * -0.00594
		//   = r * (0.15899 + r^2 * (-0.05092 + r^2 * (0.02286 + r^2 * -0.00594)))
		let r2 = r * r;
		let mut res =
			r * (P8Num::from_raw(0x0000_28B4)
				+ r2 * (P8Num::from_raw(-0x0000_0D09)
					+ r2 * (P8Num::from_raw(0x0000_05DA)
						+ r2 * P8Num::from_raw(-0x0000_0185))));
		
		if xa < ya         { res = P8Num::new(0.25) - res; }
		if x < P8Num::ZERO { res = P8Num::new(0.5)  - res; }
		if y > P8Num::ZERO { res = P8Num::new(1.0)  - res; }
		
		res
	}
	
	/// Returns the number of ones in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::from_raw(0x0001_0000).count_ones(), 1);
	/// assert_eq!(P8Num::from_raw(0x0000_1111).count_ones(), 4);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn count_ones(self) -> u32 {
		self.0.count_ones()
	}
	
	/// Returns the number of zeros in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	/// 
	/// assert_eq!(P8Num::new(0.0).count_zeros(), 32);
	/// assert_eq!(P8Num::new(-1.0).count_zeros(), 16);
	/// assert_eq!((-P8Num::EPSILON).count_zeros(), 0);
	/// assert_eq!(P8Num::MAX.count_zeros(), 1);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn count_zeros(self) -> u32 {
		self.0.count_zeros()
	}
	
	/// Returns the number of leading zeros in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).leading_zeros(), 32);
	/// assert_eq!(P8Num::new(1.0).leading_zeros(), 15);
	/// assert_eq!((-P8Num::EPSILON).leading_zeros(), 0);
	/// assert_eq!(P8Num::MAX.leading_zeros(), 1);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn leading_zeros(self) -> u32 {
		self.0.leading_zeros()
	}
	
	/// Returns the number of trailing zeros in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).trailing_zeros(), 32);
	/// assert_eq!(P8Num::new(1.0).trailing_zeros(), 16);
	/// assert_eq!((-P8Num::EPSILON).trailing_zeros(), 0);
	/// assert_eq!(P8Num::MAX.trailing_zeros(), 0);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn trailing_zeros(self) -> u32 {
		self.0.trailing_zeros()
	}
	
	/// Returns the number of leading ones in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).leading_ones(), 0);
	/// assert_eq!(P8Num::new(-1.0).leading_ones(), 16);
	/// assert_eq!((-P8Num::EPSILON).leading_ones(), 32);
	/// assert_eq!(P8Num::MAX.leading_ones(), 0);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn leading_ones(self) -> u32 {
		self.0.leading_ones()
	}
	
	/// Returns the number of trailing ones in the binary representation of `self`.
	///
	/// # Examples
	///
	/// ```
	/// use p8rs_types::p8num::P8Num;
	///
	/// assert_eq!(P8Num::new(0.0).trailing_ones(), 0);
	/// assert_eq!(P8Num::new(1.0).trailing_ones(), 0);
	/// assert_eq!((-P8Num::EPSILON).trailing_ones(), 32);
	/// assert_eq!(P8Num::MAX.trailing_ones(), 31);
	/// ```
	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline(always)]
	pub const fn trailing_ones(self) -> u32 {
		self.0.trailing_ones()
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
		self.wrapping_add(rhs)
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
		self.wrapping_sub(rhs)
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
		self.wrapping_mul(rhs)
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
		self.wrapping_div(rhs)
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
		self.wrapping_rem(rhs)
	}
}

impl RemAssign for P8Num {
	fn rem_assign(&mut self, rhs: Self) {
		*self = *self / rhs;
	}
}

impl Shl<u32> for P8Num {
	type Output = P8Num;
	
	fn shl(self, rhs: u32) -> P8Num {
		self.wrapping_shl(rhs)
	}
}

impl ShlAssign<u32> for P8Num {
	fn shl_assign(&mut self, rhs: u32) {
		*self = *self << rhs;
	}
}

impl Shr<u32> for P8Num {
	type Output = P8Num;
	
	fn shr(self, rhs: u32) -> P8Num {
		self.wrapping_shr(rhs)
	}
}

impl ShrAssign<u32> for P8Num {
	fn shr_assign(&mut self, rhs: u32) {
		*self = *self >> rhs;
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
		self.wrapping_neg()
	}
}

impl Display for P8Num {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.to_str().as_ref())
	}
}

impl Debug for P8Num {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		let raw = self.to_raw() as u32;
		write!(f, "P8Num(0x{:04X}.{:04X} = {})", raw >> 16, raw & 0xFFFF, f64::from(*self))
	}
}

impl const From<P8Num> for f32 {
	fn from(value: P8Num) -> Self {
		value.0 as f32 / (1 << 16) as f32
	}
}

impl const From<P8Num> for f64 {
	fn from(value: P8Num) -> Self {
		value.0 as f64 / (1 << 16) as f64
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TryFromError {
	/// The provided value is outside the range of representable values or is NaN.
	OutOfRange,
}

/// The error type returned when a [P8Num::from_ascii] or [P8Num::from_ascii_radix] fails.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FromAsciiError {
	/// The string contains an unexpected character
	UnexpectedChar(u8),
}

bitflags! {
    pub struct P8NumStringConversionFlags: u8 {
		/// Forces 0-padded hex format like 0x0012.3456 or 0x00123456
        const HEX = 1 << 0;
		
		/// Shift the value left 16 bits to create a 32-bit signed integer.
        const I32 = 1 << 1;
    }
}

macro_rules! impl_int_conv {
	(From $T:ty; $( $rest:tt )*) => {
		impl const From<$T> for P8Num {
			fn from(value: $T) -> P8Num {
				P8Num(i32::from(value) << 16)
			}
		}
		
		impl_int_conv!($( $rest )*);
	};
	(TryFrom $T:ty; $( $rest:tt )*) => {
		impl const TryFrom<$T> for P8Num {
			type Error = TryFromError;
			
			fn try_from(value: $T) -> Result<Self, Self::Error> {
				match value.checked_shl(16)
				           .map(TryInto::try_into) {
					Some(Ok(v)) => Ok(P8Num::from_raw(v)),
					_ => Err(TryFromError::OutOfRange),
				}
			}
		}
		
		impl_int_conv!($( $rest )*);
	};
	(Into $T:ty; $( $rest:tt )*) => {
		impl const From<P8Num> for $T {
			fn from(value: P8Num) -> Self {
				value.to_integer().into()
			}
		}
		
		impl_int_conv!($( $rest )*);
	};
	(TryInto $T:ty; $( $rest:tt )*) => {
		impl const TryFrom<P8Num> for $T {
			type Error = <i32 as TryInto<$T>>::Error;
			
			fn try_from(value: P8Num) -> Result<Self, Self::Error> {
				value.to_integer().try_into()
			}
		}
		
		impl_int_conv!($( $rest )*);
	};
	() => {};
}

impl_int_conv!(
	From    i8;    TryInto i8;
	From    i16;   Into    i16;
	TryFrom i32;   Into    i32;
	TryFrom i64;   Into    i64;
	TryFrom i128;  Into    i128;
	TryFrom isize; Into    isize;
	
	From    u8;    TryInto u8;
	TryFrom u16;   TryInto u16;
	TryFrom u32;   TryInto u32;
	TryFrom u64;   TryInto u64;
	TryFrom u128;  TryInto u128;
	TryFrom usize; TryInto usize;
);

//TODO: use feature(const_result_trait_fn)
const fn try_into_some(value: i64) -> Option<i32> {
	match value.try_into() {
		Ok(ok) => Some(ok),
		Err(_) => None,
	}
}
