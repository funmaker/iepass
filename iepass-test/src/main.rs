#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use panic_rtt_target as _;
use defmt::info;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::dma_tx_buffer;
use esp_hal::i2s::master::{Channels, Config, I2s};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    rtt_target::rtt_init_defmt!();
    
    esp_alloc::heap_allocator!(size: 64 * 1024);
    
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    
    info!("Hello World!");
    
    let test_sound = [
        0b_0000_0000_0000_0000,
        0b_1010_1010_0101_0101,
        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_0001,
        0b_0000_0000_0000_0010,
        0b_0000_0000_0000_0100,
        0b_0000_0000_0000_1000,
        0b_0000_1111_1111_0000,
        0b_0001_0000_0000_0000,
        0b_0011_0000_0000_0000,
        0b_0111_0000_0000_0000,
        0b_1111_0000_0000_0000,
    ].map(u16::to_le_bytes);
    
    let mut test_sound = test_sound
        .as_flattened()
        .iter()
        .copied()
        .cycle();
    
    
    let (descriptors, buffer) = dma_tx_buffer!(32736).unwrap().split();
    
    let i2s = I2s::new(
        peripherals.I2S0,
        peripherals.DMA_CH1,
        Config::new_tdm_philips()
            .with_sample_rate(Rate::from_hz(44100))
            .with_channels(Channels::STEREO),
    );
    
    let mut transfer = i2s
        .unwrap()
        .into_async()
        .i2s_tx
        .with_bclk(peripherals.GPIO6)
        .with_ws(peripherals.GPIO5)
        .with_dout(peripherals.GPIO7)
        .build(descriptors)
        .write_dma_circular_async(buffer)
        .unwrap();
    
    info!("Playing test pattern.");
    
    loop {
        transfer.push_with(|chunk| {
            chunk.fill_with(|| test_sound.next().unwrap());
            chunk.len()
        }).await.unwrap();
    }
}