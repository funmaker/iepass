use core::num::{ParseFloatError, ParseIntError};
use core::str::Utf8Error;
use bitflags::bitflags;
use thiserror::Error;
use p8rs_types::p8num::P8Num;

bitflags! {
    pub struct NumberConversionFlags: u8 {
		/// Read using hexadecimal notation, without requiring the "0x" prefix.
		/// Note: Non-hexadecimal characters, including '.' and '-', are taken to be '0'.
        const FORCE_HEX    = 1 << 0;
		
		/// Shift the value right 16 bits to create a 16.16 fixed-point number.
		/// This works with all formats, even booleans: true becomes 0x.0001.
        const SHIFT_16     = 1 << 1;
		
		/// When value cannot be converted to a number, return 0 instead of nothing.
        const ZERO_ON_FAIL = 1 << 2;
    }
}

fn parse_float_radix(s: &[u8], radix: u32) -> Result<f64, ParseNumberError> {
	match s.split_once(|&c| c == b'.') {
		None => Ok(i64::from_ascii_radix(s, radix).map(|val| val as f64)?),
		Some((whole_s, frac_s)) => {
			if frac_s.iter().find(|&&c| c == b'.').is_some() { return Err(ParseNumberError); }
			let whole = i64::from_ascii_radix(whole_s, radix)? as f64;
			let frac = i64::from_ascii_radix(frac_s, radix)? as f64;
			
			Ok(whole + frac / (radix.pow(frac_s.len() as u32) as f64))
		}
	}
}

fn apply_flags(num: f64, flags: NumberConversionFlags) -> f64 {
	if flags.contains(NumberConversionFlags::SHIFT_16) {
		num / 65536.0
	} else {
		num
	}
}

pub fn number_from_ascii(s: &[u8], flags: NumberConversionFlags) -> Result<P8Num, ParseNumberError> {
	let s = s.trim_ascii();
	
	let result = if let Some(s) = s.strip_prefix(b"0x") {
		parse_float_radix(s, 16)
	} else if flags.contains(NumberConversionFlags::FORCE_HEX) {
		parse_float_radix(s, 16)
	} else if let Some(s) = s.strip_prefix(b"0b") {
		parse_float_radix(s, 2)
	} else {
		core::str::from_utf8(s)
			.map_err(Into::into)
			.and_then(|s| s.parse().map_err(Into::into))
	};
	
	match result {
		Ok(num) => Ok(P8Num::new_f64(apply_flags(num, flags))),
		Err(_) if flags.contains(NumberConversionFlags::ZERO_ON_FAIL) => Ok(P8Num::ZERO),
		Err(err) => Err(err)
	}
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[error("Failed to parse number")]
pub struct ParseNumberError;

impl From<ParseIntError> for ParseNumberError {
	fn from(_: ParseIntError) -> ParseNumberError {
		ParseNumberError
	}
}

impl From<ParseFloatError> for ParseNumberError {
	fn from(_: ParseFloatError) -> ParseNumberError {
		ParseNumberError
	}
}

impl From<Utf8Error> for ParseNumberError {
	fn from(_: Utf8Error) -> ParseNumberError {
		ParseNumberError
	}
}
