use static_cell::{ConstStaticCell, StaticCell};
use esp_hal::system::Stack;
use embassy_executor::Spawner;
use esp_hal_embassy::Executor;
use anyhow::Result;
use embassy_time::{Duration, Timer};

use crate::utils::PerfFutureExt;

pub static STACK: ConstStaticCell<Stack<8192>> = ConstStaticCell::new(Stack::new());
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub fn cpu1() {
	info!("Initializing secondary executor.");
	
	let executor = EXECUTOR.init(Executor::new());
	executor.run(|spawner| spawner.must_spawn(cpu1_task(spawner)))
}

#[embassy_executor::task]
async fn cpu1_task(spawner: Spawner) {
	try_cpu1(spawner)
		.perf_trace("CPU1 task")
		.await
		.expect("Error in the cpu1 task");
}

async fn try_cpu1(_spawner: Spawner) -> Result<!> {
	info!("Spawning secondary tasks.");
	
	loop {
		Timer::after(Duration::from_millis(10)).await;
	}
}
