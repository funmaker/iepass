#![feature(never_type)]
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;

use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig, Pull};
use panic_rtt_target as _;
use anyhow::Result;

mod debounce;

use crate::debounce::Debounce;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();
    
    try_main(spawner).await.expect("Error in the main task");
}

async fn try_main(spawner: Spawner) -> Result<!> {
    info!("Initializing system.");
    
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    
    info!("Initializing allocators.");
    
    esp_alloc::heap_allocator!(size: 64 * 1024);
    
    info!("Initializing embassy.");
    
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    
    info!("Initializing peripherals.");
    
    let up_pull = InputConfig::default().with_pull(Pull::Up);
    let mut select_btn = Debounce::new(Input::new(peripherals.GPIO14, up_pull));
    let mut start_btn = Debounce::new(Input::new(peripherals.GPIO4, up_pull));
    let mut x_btn = Debounce::new(Input::new(peripherals.GPIO15, up_pull));
    let mut y_btn = Debounce::new(Input::new(peripherals.GPIO16, up_pull));
    let mut a_btn = Debounce::new(Input::new(peripherals.GPIO17, up_pull));
    let mut b_btn = Debounce::new(Input::new(peripherals.GPIO18, up_pull));
    
    
    info!("Spawning tasks.");
    
    let _ = spawner;
    
    info!("Entering main loop.");
    
    loop {
        Timer::after(Duration::from_millis(1_000 / 60)).await;
        
        if select_btn.falling_edge() { info!("select_btn"); }
        if start_btn.falling_edge() { info!("start_btn"); }
        if x_btn.falling_edge() { info!("x_btn"); }
        if y_btn.falling_edge() { info!("y_btn"); }
        if a_btn.falling_edge() { info!("a_btn"); }
        if b_btn.falling_edge() { info!("b_btn"); }
    }
}

