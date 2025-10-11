use core::num::{ParseFloatError, ParseIntError};
use p8rs_piccolo::Value;
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

fn parse_float_radix(s: &str, radix: u32) -> Result<f64, ParseNumberError> {
	match s.split_once('.') {
		None => Ok(i64::from_str_radix(s, radix).map(|val| val as f64)?),
		Some((whole_s, frac_s)) => {
			if frac_s.find('.').is_some() { return Err(ParseNumberError); }
			let whole = i64::from_str_radix(whole_s, radix)? as f64;
			let frac = i64::from_str_radix(frac_s, radix)? as f64;
			
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

pub fn number_from_string<'gc>(s: &str, flags: NumberConversionFlags) -> Result<Value<'gc>, ParseNumberError> {
	let s = s.trim();
	
	let result = if let Some(s) = s.strip_prefix("0x") {
		parse_float_radix(s, 16)
	} else if flags.contains(NumberConversionFlags::FORCE_HEX) {
		parse_float_radix(s, 16)
	} else if let Some(s) = s.strip_prefix("0b") {
		parse_float_radix(s, 2)
	} else {
		s.parse::<f64>().map_err(Into::into)
	};
	
	match result {
		Ok(num) => Ok(Value::Number(P8Num::new_f64(apply_flags(num, flags)))),
		Err(_) if flags.contains(NumberConversionFlags::ZERO_ON_FAIL) => Ok(Value::Number(P8Num::ZERO)),
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
