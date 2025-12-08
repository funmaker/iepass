use std::fmt::{Display, Formatter};

pub fn format_bytes(bytes: f64) -> impl Display {
	struct FormatBytes(f64);
	impl Display for FormatBytes {
		fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
			let bytes = self.0;
			match bytes.log2() {
				..10.0 => write!(f, "{} bytes", bytes.round() as usize),
				..20.0 => write!(f, "{:.1}KB", bytes / 1024.0),
				..30.0 => write!(f, "{:.1}MB", bytes / 1024.0 / 1024.0),
				_ => write!(f, "{:.1}GB", bytes / 1024.0 / 1024.0 / 1024.0),
			}
		}
	}
	
	FormatBytes(bytes)
}


pub fn format_millis(millis: f64) -> impl Display {
	struct FormatTime(f64);
	impl Display for FormatTime {
		fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
			let millis = self.0;
			match millis {
				..0.0001 => write!(f, "<1us"),
				..1.0 => write!(f, "{:.0}us", millis * 1000.0),
				..10.0 => write!(f, "{:.3}ms", millis),
				..100.0 => write!(f, "{:.2}ms", millis),
				..1000.0 => write!(f, "{:.1}ms", millis),
				_ => write!(f, "{:.0}ms", millis),
			}
		}
	}
	
	FormatTime(millis)
}
