use super::{FromAsciiError, P8Num};

pub fn from_ascii_dec(src: &[u8]) -> Result<P8Num, FromAsciiError> {
	if src.is_empty() {
		return Err(FromAsciiError::Empty);
	}
	
	for char in src {
		if !matches!(char, b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
			return Err(FromAsciiError::UnexpectedChar(*char));
		}
	}
	
	let float = src.as_ascii().unwrap().as_str().parse::<f64>().map_err(|_| FromAsciiError::InvalidLiteral)?;
	
	if float >= 140737488355328.0 {
		Ok(P8Num::ZERO)
	} else {
		Ok(P8Num::new_f64(float))
	}
}

pub fn from_ascii_bin(src: &[u8]) -> Result<P8Num, FromAsciiError> {
	for char in src {
		if !matches!(char, b'0'..=b'1' | b'-' | b'+' | b'.') {
			return Err(FromAsciiError::UnexpectedChar(*char));
		}
	}
	
	let (src, negative) = match src {
		[b'-', rest @ ..] => (rest, true),
		[b'+', rest @ ..] => (rest, false),
		[rest @ ..] => (rest, false),
	};
	
	const MAX_EXP: u32 = 16;
	let mut dot = false;
	let mut value = 0_u32;
	let mut exp = 0;
	
	for char in src.iter() {
		if exp >= MAX_EXP {
			break;
		}
		
		match *char {
			b'0' | b'1' => {
				value = value.wrapping_shl(1);
				if *char == b'1' {
					value += 1;
				}
				if dot { exp += 1; }
			},
			b'.' if !dot => dot = true,
			_ => return Err(FromAsciiError::UnexpectedChar(*char)),
		}
	}
	
	let mut value = value.wrapping_shl(MAX_EXP - exp).cast_signed();
	
	if negative {
		value = value.wrapping_neg();
	}
	
	Ok(P8Num::from_raw(value))
}

pub fn from_ascii_hex(src: &[u8]) -> Result<P8Num, FromAsciiError> {
	for char in src.iter() {
		if !matches!(char, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' | b'-' | b'+' | b'.') {
			return Err(FromAsciiError::UnexpectedChar(*char));
		}
	}
	
	let (src, negative) = match src {
		[b'-', rest @ ..] => (rest, true),
		[b'+', rest @ ..] => (rest, false),
		[rest @ ..] => (rest, false),
	};
	
	const MAX_EXP: u32 = 4;
	let mut dot = false;
	let mut value = 0_u32;
	let mut exp = 0;
	
	for char in src.iter() {
		if exp >= MAX_EXP {
			break;
		}
		
		match *char {
			b'0'..=b'9' => {
				value = value.wrapping_shl(4);
				value += (*char - b'0') as u32;
				if dot { exp += 1; }
			},
			b'a'..=b'f' => {
				value = value.wrapping_shl(4);
				value += (*char - b'a') as u32 + 10;
				if dot { exp += 1; }
			},
			b'A'..=b'F' => {
				value = value.wrapping_shl(4);
				value += (*char - b'A') as u32 + 10;
				if dot { exp += 1; }
			},
			b'.' if !dot => dot = true,
			_ => return Err(FromAsciiError::UnexpectedChar(*char)),
		}
	}
	
	let mut value = value.wrapping_shl((MAX_EXP - exp) * 4).cast_signed();
	
	if negative {
		value = value.wrapping_neg();
	}
	
	Ok(P8Num::from_raw(value))
}
