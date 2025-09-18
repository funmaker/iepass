use piccolo::Value;
use bitflags::bitflags;

bitflags! {
	/**
	ForceHex: Read using hexadecimal notation, without requiring the "0x" prefix.
	     Note: Non-hexadecimal characters, including '.' and '-', are taken to be '0'.
	
	Shift16: Shift the value right 16 bits to create a 16.16 fixed-point number.
	     This works with all formats, even booleans: true becomes 0x.0001.
	
	ZeroOnFail: When value cannot be converted to a number, return 0 instead of nothing.
	*/
    pub struct NumberConversionFlags: u8 {
        const NONE         = 0;
        const FORCE_HEX    = 1 << 0;
        const SHIFT_16     = 1 << 1;
        const ZERO_ON_FAIL = 1 << 2;
    }
}

fn parse_float_radix(s: &str, radix: u32) -> Result<f64, ()> {
	match s.split_once('.') {
		None => i64::from_str_radix(s, radix).map(|x| x as f64).map_err(|_| ()),
		Some((whole_s, frac_s)) => {
			if frac_s.find('.').is_some() { return Err(()); }
			let whole = i64::from_str_radix(whole_s, radix).map_err(|_| ())? as f64;
			let frac = i64::from_str_radix(frac_s, radix).map_err(|_| ())? as f64;
			
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

pub fn number_from_string<'gc>(s: &str, flags: NumberConversionFlags) -> Result<Value<'gc>, ()> {
	let s = s.trim();
	
	let result = if flags.contains(NumberConversionFlags::FORCE_HEX) {
		parse_float_radix(s, 16)
	} else if s.starts_with("0x") {
		parse_float_radix(&s[2..], 16)
	} else if s.starts_with("0b") {
		parse_float_radix(&s[2..], 2)
	} else {
		s.parse::<f64>().map_err(|_| ())
	};
	
	match result {
		Ok(num) => Ok(Value::Number(apply_flags(num, flags))),
		Err(_) if flags.contains(NumberConversionFlags::ZERO_ON_FAIL) => Ok(Value::Number(0.0)),
		Err(_) => Err(())
	}
}
