use anyhow::Result;
use embassy_futures::yield_now;
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
		let frame = fb.seq as f32 * 3.0;
		
		for y in 0..display::HEIGHT {
			for x in 0..display::WIDTH {
				fn b2i (b: bool) -> usize { if b { 1 } else { 0 } }
				
				let delay = (0..3).find_map(|d| {
					let x = (x as isize) / 6 - 2;
					let y = (y as f32 + (x as f32 / 4.0 - (frame - d as f32 * 20.0) / 15.0).sin() * 6.0) as isize / 6 - 6;
					if x < 0 || x >= 23 || y < 0 || y >= 9 {
						return None;
					}
					let value = MASK[(x as usize + y as usize * 23) % MASK.len()];
					if value == 0 { None } else { Some((d, value)) }
				});
				
				let color = match delay {
					Some((d, 1)) => Color::GRAY.linear_mul(1.0 / (d + 1) as f32),
					Some((d, 2)) => Color::BLUE.linear_mul(1.0 / (d + 1) as f32),
					_ => {
						let x = x as f32 + 0.1;
						let y = y as f32 + 0.1;
						if (
							b2i((x + frame * 0.8 + 100.0) % 16.0 > 8.0)
								+ b2i((
								y + (frame / 60.0).sin() * 30.0 / 3.0
									+ (x / 10.0 + frame / 30.0).sin() * 5.0 + 100.0
							) % 16.0 > 8.0)
						) % 2 == 0 {
							Color::MAGENTA.linear_mul((x / 10.0 + frame / 30.0).sin() * 0.4 + 0.6)
						} else {
							Color::BLACK
						}
					}
				};
				
				fb.set(x + y * display::WIDTH, color);
			}
			
			if y % 16 == 0 {
				yield_now().await;
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

const MASK: [u8; 23 * 9] = [
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 2, 0, 2, 2, 2, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0,
	0, 2, 0, 2, 2, 2, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0,
	0, 2, 0, 2, 2, 2, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
