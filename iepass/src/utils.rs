use crate::colors::Color;

const SCR_WIDTH: u16 = 160;
const SCR_HEIGHT: u16 = 128;

pub fn draw_rect(framebuffer: &mut [u16], filled: bool, x: u16, y: u16, w: u16, h: u16, color: Color) {
	let x = x.min(SCR_WIDTH);
	let y = y.min(SCR_HEIGHT);
	let w = w.min(SCR_WIDTH - x);
	let h = h.min(SCR_HEIGHT - y);
	
	for row in y..(y + h) {
		if filled || (row == y) || (row == y + h - 1) {
			framebuffer[(row * SCR_WIDTH + x) as usize..(row * SCR_WIDTH + x + w) as usize].fill(color.into());
		} else {
			framebuffer[(row * SCR_WIDTH + x) as usize] = color.into();
			framebuffer[(row * SCR_WIDTH + x + w - 1) as usize] = color.into();
		}
	}
}
