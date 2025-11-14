
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Color(u16);

#[allow(dead_code)]
impl Color {
	pub const BLACK: Color = Color::new(0, 0, 0);
	pub const RED: Color = Color::new(255, 0, 0);
	pub const YELLOW: Color = Color::new(255, 255, 0);
	pub const GREEN: Color = Color::new(0, 255, 0);
	pub const TEAL: Color = Color::new(0, 255, 255);
	pub const BLUE: Color = Color::new(0, 0, 255);
	pub const MAGENTA: Color = Color::new(255, 0, 255);
	pub const GRAY: Color = Color::new(127, 127, 127);
	pub const WHITE: Color = Color::new(255, 255, 255);
	
	pub const fn new(r: u8, g: u8, b: u8) -> Self {
		Self(
			((r & 0b1111_1000) as u16) << 8 |
				((g & 0b1111_1100) as u16) << 3 |
				((b & 0b1111_1000) as u16) >> 3
		)
	}
	
	pub fn from_raw(inner: u16) -> Self {
		Self(inner)
	}
	
	pub fn rgb(&self) -> (u8, u8, u8) {
		(
			(self.0 >> 8) as u8 & 0b1111_1000,
			(self.0 >> 3) as u8 & 0b1111_1100,
			(self.0 << 3) as u8 & 0b1111_1000,
		)
	}
	
	pub fn as_u16(&self) -> u16 {
		self.0
	}
	
	pub fn linear_mul(self, scale: f32) -> Self {
		let (r, g, b) = self.rgb();
		Self::new(
			(r as f32 * scale) as u8,
			(g as f32 * scale) as u8,
			(b as f32 * scale) as u8,
		)
	}
}

impl Into<u16> for Color {
	fn into(self) -> u16 {
		self.as_u16()
	}
}
