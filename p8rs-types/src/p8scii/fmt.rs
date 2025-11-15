use core::fmt;
use core::fmt::{Formatter, Display as StdDisplay, Debug as StdDebug};
use crate::p8scii;
use crate::p8scii::to_char;

pub struct Display<T>(pub T);

impl StdDisplay for Display<u8> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", to_char(self.0))?;
		
		Ok(())
	}
}

impl StdDebug for Display<u8> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "\'")?;
		
		for char in p8scii::to_iter(p8scii::escape([self.0])) {
			write!(f, "{}", char)?;
		}
		
		write!(f, "\'")?;
		
		Ok(())
	}
}

impl StdDisplay for Display<&[u8]> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for &byte in self.0 {
			write!(f, "{}", to_char(byte))?;
		}
		
		Ok(())
	}
}

impl StdDebug for Display<&[u8]> {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		write!(fmt, "\"")?;
		
		for char in p8scii::to_iter(p8scii::escape(self.0.iter().copied())) {
			write!(fmt, "{}", char)?;
		}
		
		write!(fmt, "\"")?;
		
		Ok(())
	}
}
