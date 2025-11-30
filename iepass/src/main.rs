#![allow(incomplete_features)]
#![feature(never_type)]
#![feature(iter_array_chunks)]
#![feature(generic_const_exprs)]
#![no_std]
#![no_main]
#![deny(clippy::mem_forget, reason = "mem::forget is generally not safe to do with esp_hal types, especially those holding buffers for the duration of a data transfer.")]

extern crate alloc;
#[macro_use] extern crate p8rs;

use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use embassy_executor::Spawner;
use esp_hal::gpio::{Input, InputConfig, Level, Output, Pull};
use esp_hal::system::CpuControl;
use panic_rtt_target as _;
use anyhow::{anyhow, Result};
use esp_hal::{gpio, psram};
use rtt_target::ChannelMode;
use p8rs::colors::Color;
use p8rs::vm::memory::machine_state::Palette;
use p8rs::vm::{palette, P8rs};

mod calib;
mod peripherials;
mod tasks;
mod utils;

use peripherials::{Debounce, Display, Speaker, Analog, SpiBus, Touch, display};
use tasks::display::FRAMEBUFFER_MANAGER;
use calib::Calib;
use utils::{PSRAM_ALLOCATOR, perf, PerfFutureExt};

// static KUTASAN: &[u8] = include_bytes!("../../assets/kutasan.pcm");

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
    
    let config = esp_hal::Config::default()
        .with_cpu_clock(CpuClock::max())
        .with_psram(psram::PsramConfig {
            ram_frequency: psram::SpiRamFreq::Freq40m,
            core_clock: Some(psram::SpiTimingConfigCoreClock::SpiTimingConfigCoreClock80m),
            size: psram::PsramSize::Size(2097152),
            ..Default::default()
        });
    let calib = Calib::default();
    let peripherals = esp_hal::init(config);
    
    info!("Initializing allocators.");
    
    // SRAM global allocator
    esp_alloc::heap_allocator!(size: 140 * 1024);
    
    // PSRAM custom allocator
    {
        let (start, size) = psram::psram_raw_parts(&peripherals.PSRAM);
        unsafe {
            PSRAM_ALLOCATOR.add_region(esp_alloc::HeapRegion::new(
                start,
                size,
                esp_alloc::MemoryCapability::External.into(),
            ));
        }
    }
    
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
    
    let mut analog = Analog::new(
        peripherals.ADC1,
        peripherals.GPIO8,
        peripherals.GPIO9,
        calib.analog_deadzone,
        calib.analog,
    );
    let _analog_btn = Debounce::new(Input::new(peripherals.GPIO3, up_pull));
    
    let _sd_cs = Output::new(peripherals.GPIO38, Level::High, gpio::OutputConfig::default());
    
    let spi_bus = SpiBus::new(
        peripherals.SPI3,
        peripherals.GPIO35,
        peripherals.GPIO36,
        peripherals.GPIO37,
        peripherals.DMA_CH2,
    )?;
    
    let mut touch = Touch::new(
        &spi_bus,
        peripherals.GPIO40,
        peripherals.GPIO39,
        calib.touch,
    );
    
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
    
    info!("Creating Pico-8");
    
    let mut pico8 = P8rs::new_in(&PSRAM_ALLOCATOR)?;
    
    info!("Loading mandelbrot.p8");
    
    pico8.load_cartridge(include_bytes!("../../lua/mandelbrot.p8"))?;
    
    info!("Entering main loop.");
    
    let mut fbs = FRAMEBUFFER_MANAGER.producer();
    
    loop {
        if let Some((x, y)) = touch.read(100, 100).await? { info!("Touch: {} {}", x, y); }
        if dbg_btn.falling_edge() { perf::dump_perf()?; }
        if x_btn.falling_edge() { info!("x_btn"); }
        if y_btn.falling_edge() { info!("y_btn"); }
        if a_btn.falling_edge() { info!("a_btn"); }
        if b_btn.falling_edge() { info!("b_btn"); }
        if select_btn.falling_edge() { info!("select_btn"); }
        if start_btn.falling_edge() {
            info!("start_btn");
            
            info!("{}", analog.read(100));
            
            // speaker.play(&*KUTASAN).await?;
            speaker.reset().await?;
        }
        
        pico8.run()?; // TODO: result.requested_fps
        
        let runtime = pico8.runtime();
        let screen_palette = *runtime.memory.machine_state().palette(Palette::Screen);
        let map_color = |color: u8| -> Color {
            assert!(color < 16);
            palette::color_from_index(screen_palette[color as usize])
        };
        
        let mut fb = fbs.get_empty().await;
        let screen = runtime.memory.screen();
        let pixels = screen.iter()
                           .map(|byte| [map_color(*byte & 0x0F), map_color(*byte >> 4)])
                           .flatten()
                           .enumerate();
        for (id, pixel) in pixels {
            let x = id % 128;
            let y = id / 128;
            
            fb.as_raw_pixels()[(display::WIDTH as usize - 128) / 2 + x + y * display::WIDTH as usize] = pixel.as_u16().to_be_bytes();
        }
        fbs.put_drawn(fb).await;
    }
}