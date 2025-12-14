#![allow(incomplete_features)]
#![feature(never_type)]
#![feature(iter_array_chunks)]
#![feature(generic_const_exprs)]
#![feature(allocator_api)]
#![no_std]
#![no_main]
#![deny(clippy::mem_forget, reason = "mem::forget is generally not safe to do with esp_hal types, especially those holding buffers for the duration of a data transfer.")]

#[macro_use] extern crate p8rs_log;
extern crate alloc;

use core::mem::MaybeUninit;
use panic_rtt_target as _;
use rtt_target::ChannelMode;
use esp_hal::{gpio, psram, ram};
use esp_hal::timer::timg;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, Pull};
use embassy_executor::Spawner;
use anyhow::Result;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use p8rs::colors::Color;
use p8rs::vm::memory::machine_state::Palette;
use p8rs::vm::{palette, P8rs};

mod calib;
mod peripherials;
mod tasks;
mod utils;
mod callbacks;

use peripherials::{Debounce, Display, Speaker, Analog, SpiBus, Touch, Controller, display};
use tasks::display::FRAMEBUFFER_MANAGER;
use calib::Calib;
use utils::{PSRAM_ALLOCATOR, PerfFutureExt};
use callbacks::IepassCallbacks;

// static KUTASAN: &[u8] = include_bytes!("../../assets/kutasan.pcm");

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
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
            size: psram::PsramSize::AutoDetect,
            ..Default::default()
        });
    let calib = Calib::default();
    let peripherals = esp_hal::init(config);
    
    info!("Initializing allocators.");
    
    #[allow(static_mut_refs)]
    unsafe {
        use esp_alloc::MemoryCapability;
        
        #[ram(reclaimed)]
        static mut HEAP_RECLAIMED: MaybeUninit<[u8; 73744]> = MaybeUninit::uninit();
        static mut HEAP_EXTRA: MaybeUninit<[u8; 200 * 1024]> = MaybeUninit::uninit();
        
        let (sram_start, sram_size) = (HEAP_RECLAIMED.as_mut_ptr() as *mut u8, size_of_val(&HEAP_RECLAIMED));
        let (sram_ex_start, sram_ex_size) = (HEAP_EXTRA.as_mut_ptr() as *mut u8, size_of_val(&HEAP_EXTRA));
        let (psram_start, psram_size) = psram::psram_raw_parts(&peripherals.PSRAM);
        
        info!("SRAM heap initialized at 0x{:08x}..0x{:08x} ({} bytes)", sram_start as usize, sram_start as usize + sram_size, sram_size);
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(sram_start, sram_size, MemoryCapability::Internal.into()));
        
        info!("SRAM heap initialized at 0x{:08x}..0x{:08x} ({} bytes)", sram_ex_start as usize, sram_ex_start as usize + sram_ex_size, sram_ex_size);
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(sram_ex_start, sram_ex_size, MemoryCapability::Internal.into()));
        
        info!("PSRAM heap initialized at 0x{:08x}..0x{:08x} ({} bytes)", psram_start as usize, psram_start as usize + psram_size, psram_size);
        PSRAM_ALLOCATOR.add_region(esp_alloc::HeapRegion::new(psram_start, psram_size, MemoryCapability::External.into()));
    }
    
    info!("Initializing embassy.");
    
    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = timg::TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);
    
    info!("Initializing peripherals.");
    
    let up_pull = InputConfig::default().with_pull(Pull::Up);
    let dbg_btn = Debounce::new(Input::new(peripherals.GPIO0, up_pull));
    
    let controller = Controller::new(
        Debounce::new(Input::new(peripherals.GPIO14, up_pull)),
        Debounce::new(Input::new(peripherals.GPIO4, up_pull)),
        Debounce::new(Input::new(peripherals.GPIO15, up_pull)),
        Debounce::new(Input::new(peripherals.GPIO16, up_pull)),
        Debounce::new(Input::new(peripherals.GPIO17, up_pull)),
        Debounce::new(Input::new(peripherals.GPIO18, up_pull)),
        Debounce::new(Input::new(peripherals.GPIO3, up_pull)),
        Analog::new(
            peripherals.ADC1,
            peripherals.GPIO8,
            peripherals.GPIO9,
            calib.analog_deadzone,
            calib.analog,
        ),
    );
    
    let _sd_cs = Output::new(peripherals.GPIO38, Level::High, gpio::OutputConfig::default());
    
    let spi_bus = SpiBus::new(
        peripherals.SPI3,
        peripherals.GPIO47,
        peripherals.GPIO48,
        peripherals.GPIO45,
        peripherals.DMA_CH2,
    )?;
    
    let mut touch = Touch::new(
        &spi_bus,
        peripherals.GPIO40,
        peripherals.GPIO39,
        calib.touch,
    );
    
    let _speaker = Speaker::new(
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
    
    info!("Initializing second cpu.");
    
    esp_rtos::start_second_core_with_stack_guard_offset(
        peripherals.CPU_CTRL,
        software_interrupt.software_interrupt1,
        tasks::cpu1::STACK.take(),
        None,
        || tasks::cpu1(dbg_btn),
    );
    
    info!("Spawning tasks.");
    
    spawner.spawn(tasks::display(display))?;
    spawner.spawn(tasks::draw(true))?;
    
    info!("Creating Pico-8");
    
    let mut pico8 = P8rs::new_in(&PSRAM_ALLOCATOR)?;
    pico8.set_callbacks(IepassCallbacks::new(controller));
    
    info!("Loading mener.p8");
    
    pico8.load_cartridge(include_bytes!("../../lua/mener.p8"))?;
    
    info!("Entering main loop.");
    
    let mut fbs = FRAMEBUFFER_MANAGER.producer();
    
    loop {
        if let Some((x, y)) = touch.read(100, 100).await? { info!("Touch: {} {}", x, y); }
        
        pico8.run()?; // TODO: result.requested_fps
        
        let runtime = pico8.runtime();
        let screen_palette = *runtime.memory.machine_state().palette(Palette::Screen);
        let map_color = |color: u8| -> Color {
            assert!(color < 16);
            palette::color_from_index(screen_palette[color as usize])
        };
        
        let mut fb = fbs.get_empty().await;
        fb.fill(Color::BLACK);
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