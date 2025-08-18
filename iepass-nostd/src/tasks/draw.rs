use anyhow::Result;
use micromath::F32Ext;

use crate::peripherials::display;
use crate::tasks::display::FRAMEBUFFER_MANAGER;
use crate::utils::{Color, Framebuffer, PerfFutureExt};

pub const DRAW_TASKS_MAX: usize = 2;

#[embassy_executor::task(pool_size = DRAW_TASKS_MAX)]
pub async fn draw(first: bool) {
	try_draw()
		.perf_trace(if first { "Draw Task 1" } else { "Draw Task 2" })
		.await
		.expect("Error in the draw task");
}

async fn try_draw() -> Result<!> {
	let mut producer = FRAMEBUFFER_MANAGER.producer();
	loop {
		let mut fb = producer.get_empty().await;
		let frame = fb.seq as f32;
		
		for rowi in -1..display::HEIGHT as i16 / 16 + 1 {
			let row = rowi as f32;
			
			for coli in 0..display::WIDTH as i16 / 16 + 2 {
				let col = coli as f32 - frame / 30.0 % 2.0;
				let height = (frame / 120.0 + col * 1.2).sin();
				let color = if rowi.rem_euclid(2) == coli.rem_euclid(2) { Color::MAGENTA.linear_mul(0.75 - height * 0.25) } else { Color::BLACK };
				
				draw_rect(&mut fb, true, col * 16.0, row * 16.0 + height * 4.0, 16.0, 16.0, color);
			}
		}
		producer.put_drawn(fb).await;
	}
}

pub fn draw_rect(framebuffer: &mut Framebuffer, filled: bool, x: f32, y: f32, mut w: f32, mut h: f32, color: Color) {
	if x < 0.0 { w += x; }
	if y < 0.0 { h += y; }
	
	let x = x.min(display::WIDTH as f32).max(0.0) as u16;
	let y = y.min(display::HEIGHT as f32).max(0.0) as u16;
	let w = w.min((display::WIDTH - x) as f32).max(0.0) as u16;
	let h = h.min((display::HEIGHT - y) as f32).max(0.0) as u16;
	
	if w <= 0 || h <= 0 {
		return;
	}
	
	for row in y..(y + h) {
		if filled || (row == y) || (row == y + h - 1) {
			framebuffer.fill_line(row * display::WIDTH + x, w, color);
		} else {
			framebuffer.set(row * display::WIDTH + x, color);
			framebuffer.set(row * display::WIDTH + x + w - 1, color);
		}
	}
}
