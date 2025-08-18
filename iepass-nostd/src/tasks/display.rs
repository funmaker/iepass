use anyhow::Result;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::peripherials::display::{Display};
use crate::utils::framebuffer::{static_framebuffer, Framebuffer};
use crate::utils::PerfFutureExt;

pub static FRAMES_READY: Channel<CriticalSectionRawMutex, Framebuffer, 1> = Channel::new();
pub static FRAMES_EMPTY: Channel<CriticalSectionRawMutex, Framebuffer, 1> = Channel::new();

type Delay = embassy_time::Delay;

#[embassy_executor::task]
pub async fn display(display: Display<'static, Delay>) {
	try_display(display).perf_trace("Display Task")
	                    .await
	                    .expect("Error in the display task");
}

async fn try_display(mut display: Display<'static, Delay>) -> Result<!> {
	FRAMES_EMPTY.send(static_framebuffer!()).await;
	FRAMES_EMPTY.send(static_framebuffer!()).await;
	
	loop {
		let frame = FRAMES_READY.receive().await;
		let frame = display.draw_async(frame)
		                   .perf_trace("SPI")
		                   .await
		                   .map_err(|(err, _)| err)?;
		FRAMES_EMPTY.send(frame).await;
	}
}
