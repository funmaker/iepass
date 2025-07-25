#![feature(try_blocks)]

use std::time::Instant;
use iepass_core::rle;
use thiserror::Error;
use embedded_io::{Read, ReadExactError};
use st7735_lcd::{Orientation, ST7735};
use embedded_graphics_core::pixelcolor::raw::RawU16;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::primitives::Rectangle;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Gpio0, PinDriver, Pull};
use esp_idf_svc::hal::spi::{config, Dma, SpiConfig, SpiDeviceDriver};
use esp_idf_svc::hal::spi::config::DriverConfig;

mod debounce;

use debounce::Debounce;

#[cfg(feature = "bad-apple")] static VIDEO: &[u8] = include_bytes!("../../assets/BadApple.smol");
#[cfg(not(feature = "bad-apple"))] static VIDEO: &[u8] = include_bytes!("../../assets/XD.smol");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    let peripherals = Peripherals::take().unwrap();
    
    let mut select_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio1)?).with_pull(Pull::Up)?;
    let mut start_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio19)?).with_pull(Pull::Up)?;
    let mut a_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio14)?).with_pull(Pull::Up)?;
    let mut b_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio13)?).with_pull(Pull::Up)?;
    let mut x_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio12)?).with_pull(Pull::Up)?;
    let mut y_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio11)?).with_pull(Pull::Up)?;
    
    let mut display = {
        let rgb = true;
        let inverted = false;
        let width = 160;
        let height = 128;
        
        let rst = PinDriver::output(peripherals.pins.gpio42)?;
        let a0 = PinDriver::output(peripherals.pins.gpio41)?;
        let sda = peripherals.pins.gpio40;
        let sck = peripherals.pins.gpio39;
        
        let spi = SpiDeviceDriver::new_single(
            peripherals.spi2,
            sck,
            sda,
            None::<Gpio0>,
            None::<Gpio0>,
            &DriverConfig {
                dma: Dma::Auto(128 * 160 * 2),
                intr_flags: Default::default(),
            },
            &SpiConfig::new().baudrate(30.MHz().into())
        )?;
        
        ST7735::new(spi, a0, rst, rgb, inverted, width, height)
    };
    
    display.init(&mut FreeRtos).map_err(|_| DisplayError::InitError)?;
    display.set_orientation(&Orientation::Landscape).map_err(|_| DisplayError::SetOrientationError)?;
    display.set_offset(1, 2); // No idea why its needed
    display.clear(Rgb565::MAGENTA).map_err(|_| DisplayError::ClearError)?;

    log::info!("Hello, world!");
    
    let mut framebuffer = vec![0; 128 * 160];
    
    loop {
        FreeRtos::delay_ms(10);
        
        if select_btn.falling_edge() {
            log::info!("select");
            display.clear(Rgb565::MAGENTA).map_err(|_| DisplayError::ClearError)?;
            display.fill_solid(
                &Rectangle::new(Point::new(0, 0), Size::new(160, 128)),
                Rgb565::MAGENTA,
            ).map_err(|_| DisplayError::DrawError)?;
        }
        if start_btn.falling_edge() {
            log::info!("start");
            
            let start = Instant::now();
            let mut frames = 0;
            let mut parts = (0.0, 0.0, 0.0);
            let mut decoder = rle::Decoder::new(VIDEO);
            let mut row = [0; 160];
            display.set_address_window(0, 0, 159, 127).map_err(|_| DisplayError::SetOrientationError)?;
            
            'outer: for _ in 0.. {
                frames += 1;
                
                let now = Instant::now();
                for y in 0..128 {
                    if start_btn.falling_edge() {
                        break 'outer;
                    }
                    
                    match decoder.read_exact(&mut row) {
                        Err(ReadExactError::UnexpectedEof) => break 'outer,
                        result => result?,
                    }
                    
                    for x in 0..160 {
                        let color = row[x];
                        framebuffer[x + y * 160] = RawU16::from(Rgb565::new(
                            ((color as u16) * (1 << 5) / 256) as u8,
                            ((color as u16) * (1 << 6) / 256) as u8,
                            ((color as u16) * (1 << 5) / 256) as u8,
                        )).into_inner();
                    }
                }
                
                parts.0 += now.elapsed().as_secs_f32();
                let now = Instant::now();
                
                display.write_pixels_buffered(framebuffer.iter().copied()).map_err(|_| DisplayError::DrawError)?;
                
                parts.1 += now.elapsed().as_secs_f32();
                let now = Instant::now();
                
                FreeRtos::delay_ms(1);
                
                parts.2 += now.elapsed().as_secs_f32();
            }
            
            log::info!("{:.2} FPS (~{} ms)",
                       frames as f32 / start.elapsed().as_secs_f32(),
                       start.elapsed().as_millis() as u32 / frames);
            
            log::info!("{:.2} ms | {:.2} ms | {:.2} ms",
                       parts.0 * 1000.0 / frames as f32,
                       parts.1 * 1000.0 / frames as f32,
                       parts.2 * 1000.0 / frames as f32);
            
            log::info!("start done");
        }
        if a_btn.falling_edge() {
            log::info!("a");
            display.fill_solid(
                &Rectangle::new(Point::new(16, 128 - 48), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::DrawError)?;
        }
        if b_btn.falling_edge() {
            log::info!("b");
            display.fill_solid(
                &Rectangle::new(Point::new(160 - 48, 128 - 48), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::DrawError)?;
        }
        if x_btn.falling_edge() {
            log::info!("x");
            display.fill_solid(
                &Rectangle::new(Point::new(16, 16), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::DrawError)?;
        }
        if y_btn.falling_edge() {
            log::info!("y");
            display.fill_solid(
                &Rectangle::new(Point::new(160 - 48, 16), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::DrawError)?;
        }
    }
}

#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Failed to initialize display")]
    InitError,
    #[error("Failed to clear display")]
    ClearError,
    #[error("Failed to set orientation")]
    SetOrientationError,
    #[error("Failed to draw a rectangle")]
    DrawError,
}
