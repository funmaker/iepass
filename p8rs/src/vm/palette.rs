use crate::colors::Color;

pub const PALETTE: [Color; 32] = [
	Color::new(0, 0, 0),       // 00 - 0x00 - black
	Color::new(29, 43, 83),    // 01 - 0x01 - dark-blue
	Color::new(126, 37, 83),   // 02 - 0x02 - dark-purple
	Color::new(0, 135, 81),    // 03 - 0x03 - dark-green
	Color::new(171, 82, 54),   // 04 - 0x04 - brown
	Color::new(95, 87, 79),    // 05 - 0x05 - dark-grey
	Color::new(194, 195, 199), // 06 - 0x06 - light-grey
	Color::new(255, 241, 232), // 07 - 0x07 - white
	Color::new(255, 0, 77),    // 08 - 0x08 - red
	Color::new(255, 163, 0),   // 09 - 0x09 - orange
	Color::new(255, 236, 39),  // 10 - 0x0a - yellow
	Color::new(0, 228, 54),    // 11 - 0x0b - green
	Color::new(41, 173, 255),  // 12 - 0x0c - blue
	Color::new(131, 118, 156), // 13 - 0x0d - lavender
	Color::new(255, 119, 168), // 14 - 0x0e - pink
	Color::new(255, 204, 170), // 15 - 0x0f - light-peach
	Color::new(41, 24, 20),    // 16 - 0x10 - brownish-black
	Color::new(17, 29, 53),    // 17 - 0x11 - darker-blue
	Color::new(66, 33, 54),    // 18 - 0x12 - darker-purple
	Color::new(18, 83, 89),    // 19 - 0x13 - blue-green
	Color::new(116, 47, 41),   // 20 - 0x14 - dark-brown
	Color::new(73, 51, 59),    // 21 - 0x15 - darker-grey
	Color::new(162, 136, 121), // 22 - 0x16 - medium-grey
	Color::new(243, 239, 125), // 23 - 0x17 - light-yellow
	Color::new(190, 18, 80),   // 24 - 0x18 - dark-red
	Color::new(255, 108, 36),  // 25 - 0x19 - dark-orange
	Color::new(168, 231, 46),  // 26 - 0x1a - lime-green
	Color::new(0, 181, 67),    // 27 - 0x1b - medium-green
	Color::new(6, 90, 181),    // 28 - 0x1c - true-blue
	Color::new(117, 70, 101),  // 29 - 0x1d - mauve
	Color::new(255, 110, 89),  // 30 - 0x1e - dark-peach
	Color::new(255, 157, 129), // 31 - 0x1f - peach
];

pub fn color_from_index(index: u8) -> Color {
	let nib = index & 0x0F;
	if index < 128 {
		PALETTE[nib as usize]
	} else {
		PALETTE[nib as usize + 16]
	}
}
