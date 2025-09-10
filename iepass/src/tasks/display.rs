use anyhow::Result;
use defmt::info;

use crate::peripherials::display::{Display};
use crate::utils::framebuffer::{static_framebuffer, FramebufferManager};
use crate::utils::{FpsCounter, PerfFutureExt};

pub static FRAMEBUFFER_MANAGER: FramebufferManager = FramebufferManager::new();

type Delay = embassy_time::Delay;

#[embassy_executor::task]
pub async fn display(display: Display<'static, Delay>) {
	try_display(display)
		.perf_trace("Display Task")
		.await
		.expect("Error in the display task");
}

async fn try_display(mut display: Display<'static, Delay>) -> Result<!> {
	let mut fps = FpsCounter::<10>::new();
	let mut next_seq = 1;
	let mut last_seq = 0;
	
	FRAMEBUFFER_MANAGER.put_empty(static_framebuffer!(next_seq)).await;
	
	loop {
		let mut frame = FRAMEBUFFER_MANAGER.get_drawn().await;
		
		if frame.seq > last_seq {
			display.draw_async(&mut frame)
			       .perf_trace("SPI")
			       .await?;
			fps.tick();
		} else {
			defmt::warn!("Late frame {} <= {}, dropping...", frame.seq, last_seq);
		}
		
		last_seq = frame.seq;
		frame.seq = next_seq;
		next_seq += 1;
		
		FRAMEBUFFER_MANAGER.put_empty(frame).await;
		
		if next_seq % 100 == 0 {
			info!("FPS: {}", fps.fps());
		}
	}
}
