use embassy_time::{Duration, Timer};
use anyhow::Result;

use crate::peripherials::Debounce;
use crate::utils::{perf, PerfFutureExt};

#[embassy_executor::task]
pub async fn perf(button: Debounce<'static>) {
	try_perf(button)
		.perf_trace("Perf Task")
		.await
		.expect("Error in the perf task");
}

async fn try_perf(mut button: Debounce<'static>) -> Result<!> {
	loop {
		Timer::after(Duration::from_millis(10)).await;
		
		if button.falling_edge() { perf::dump_perf()?; }
	}
}
