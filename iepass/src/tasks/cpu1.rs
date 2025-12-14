use static_cell::{ConstStaticCell, StaticCell};
use esp_hal::system::Stack;
use embassy_executor::Spawner;
use anyhow::Result;
use embassy_time::{Duration, Timer};
use esp_rtos::embassy::Executor;

use crate::peripherials::Debounce;
use crate::tasks;
use crate::utils::PerfFutureExt;

pub static STACK: ConstStaticCell<Stack<8192>> = ConstStaticCell::new(Stack::new());
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub fn cpu1(dbg_btn: Debounce<'static>) {
	info!("Initializing secondary executor.");
	
	let executor = EXECUTOR.init(Executor::new());
	executor.run(|spawner| spawner.must_spawn(cpu1_task(spawner, dbg_btn)))
}

#[embassy_executor::task]
async fn cpu1_task(spawner: Spawner, dbg_btn: Debounce<'static>) {
	try_cpu1(spawner, dbg_btn)
		.perf_trace("CPU1 task")
		.await
		.expect("Error in the cpu1 task");
}

async fn try_cpu1(spawner: Spawner, dbg_btn: Debounce<'static>) -> Result<!> {
	info!("Spawning secondary tasks.");
	
	spawner.spawn(tasks::perf(dbg_btn))?;
	
	loop {
		Timer::after(Duration::from_millis(10)).await;
	}
}
