#![feature(never_type)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![no_std]
#![no_main]
#![deny(clippy::mem_forget, reason = "mem::forget is generally not safe to do with esp_hal types, especially those holding buffers for the duration of a data transfer.")]

extern crate alloc;

use alloc::vec::Vec;
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::system::CpuControl;
use panic_rtt_target as _;
use anyhow::{anyhow, Result};
use rtt_target::ChannelMode;

mod calib;
mod peripherials;
mod tasks;
mod utils;

use peripherials::{Debounce, Display, Speaker};
use calib::Calib;
use utils::{perf, PerfFutureExt};

static KUTASAN: &[u8] = include_bytes!("../../assets/kutasan.pcm");

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let channels = rtt_target::rtt_init! {
        up: {
            0: { size: 4096, mode: ChannelMode::BlockIfFull, name: "defmt" }
            // 1: { size: 4096, name: "perf" }
        }
        down: {
            0: { size: 1024, name: "stdin" }
        }
    };
    
    rtt_target::set_defmt_channel(channels.up.0);
    // perf::set_channel(channels.up.1);
    
    try_main(spawner).perf_trace("Main Task")
                     .await
                     .expect("Error in the main task");
}

async fn try_main(spawner: Spawner) -> Result<!> {
    info!("Initializing system.");
    
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let calib = Calib::default();
    let peripherals = esp_hal::init(config);
    
    info!("Initializing allocators.");
    
    esp_alloc::heap_allocator!(size: 64 * 1024);
    
    info!("Initializing embassy.");
    
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    
    info!("Initializing second cpu.");
    
    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control.start_app_core(tasks::cpu1::STACK.take(), tasks::cpu1)
                            .map_err(|err| anyhow!("{:?}", err))?;
    
    info!("Initializing peripherals.");
    
    let up_pull = InputConfig::default().with_pull(Pull::Up);
    
    let mut dbg_btn = Debounce::new(Input::new(peripherals.GPIO0, up_pull));
    let mut select_btn = Debounce::new(Input::new(peripherals.GPIO14, up_pull));
    let mut start_btn = Debounce::new(Input::new(peripherals.GPIO4, up_pull));
    let mut x_btn = Debounce::new(Input::new(peripherals.GPIO15, up_pull));
    let mut y_btn = Debounce::new(Input::new(peripherals.GPIO16, up_pull));
    let mut a_btn = Debounce::new(Input::new(peripherals.GPIO17, up_pull));
    let mut b_btn = Debounce::new(Input::new(peripherals.GPIO18, up_pull));
    
    let mut speaker = Speaker::new(
        peripherals.I2S0,
        peripherals.GPIO6,
        peripherals.GPIO5,
        peripherals.GPIO7,
        peripherals.DMA_CH1,
    )?;
    
    let display = Display::new(
        peripherals.SPI2,
        peripherals.DMA_CH0,
        peripherals.GPIO12,
        peripherals.GPIO11,
        peripherals.GPIO13,
        peripherals.GPIO10,
        calib.screen_offset,
        embassy_time::Delay,
    ).await?;
    
    info!("Spawning tasks.");
    
    spawner.spawn(tasks::display(display))?;
    spawner.spawn(tasks::draw(true))?;
    
    info!("Entering main loop.");
    
    loop {
        if dbg_btn.falling_edge() { perf::dump_perf(); }
        if x_btn.falling_edge() { info!("x_btn"); }
        if y_btn.falling_edge() { info!("y_btn"); }
        if a_btn.falling_edge() { info!("a_btn"); }
        if b_btn.falling_edge() { info!("b_btn"); }
        if select_btn.falling_edge() { info!("select_btn"); }
        if start_btn.falling_edge() {
            info!("start_btn");
            
            speaker.play(&*KUTASAN).await?;
            speaker.reset().await?;
        }
    }
}