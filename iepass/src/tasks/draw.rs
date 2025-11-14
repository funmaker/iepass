use anyhow::Result;
use p8rs::colors::Color;

use crate::tasks::display::FRAMEBUFFER_MANAGER;
use crate::utils::PerfFutureExt;

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
		
		fb.fill(Color::new(fb.seq as u8, fb.seq as u8, fb.seq as u8));
		
		producer.put_drawn(fb).await;
	}
}