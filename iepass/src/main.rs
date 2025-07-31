#![feature(try_blocks)]

use std::time::Instant;
use iepass_core::rle;
use thiserror::Error;
use embedded_io::{Read, ReadExactError};
use st7735_lcd::{Orientation, ST7735};
use esp_idf_svc::hal::adc::oneshot::AdcDriver;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver, Pull};
use esp_idf_svc::hal::spi::{Dma, SpiConfig, SpiDeviceDriver};
use esp_idf_svc::hal::spi::config::DriverConfig;

mod debounce;
mod analog;
mod colors;

use debounce::Debounce;
use analog::Analog;
use colors::Color;

// == Sound ==
//    LRC:  5
//   RCLK:  6
//    DIN:  7
// == Screen ==
//    RST: 10
//    SDA: 11
//    CLK: 12
//    A0:  13
// == Buttons ==
//  Start:  4
// Select: 14
//      X: 15
//      Y: 16
//      A: 17
//      B: 18
// == Analog ==
//      X: 19
//      Y: 20
//    BTN: 21
// == SD Card ==
//   MOSI: 35
//    CLK: 36
//   MISO: 37
//     CS: 38
// == Touch ==
//   MOSI: 35
//    CLK: 36
//   MISO: 37
//    IQR: 39
//     CS: 40


#[cfg(feature = "bad-apple")] static VIDEO: &[u8] = include_bytes!("../../assets/BadApple.smol");
#[cfg(not(feature = "bad-apple"))] static VIDEO: &[u8] = include_bytes!("../../assets/XD.smol");

const SCR_WIDTH: usize = 160;
const SCR_HEIGHT: usize = 128;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    let peripherals = Peripherals::take().unwrap();
    
    let mut select_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio14)?).with_pull(Pull::Up)?;
    let mut start_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio4)?).with_pull(Pull::Up)?;
    let mut x_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio15)?).with_pull(Pull::Up)?;
    let mut y_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio16)?).with_pull(Pull::Up)?;
    let mut a_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio17)?).with_pull(Pull::Up)?;
    let mut b_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio18)?).with_pull(Pull::Up)?;
    
    let analog_adc = AdcDriver::new(peripherals.adc2)?;
    let mut analog_x = Analog::new(&analog_adc, peripherals.pins.gpio19)?;
    let mut analog_y = Analog::new(&analog_adc, peripherals.pins.gpio20)?;
    let mut analog_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio21)?).with_pull(Pull::Up)?;
    
    let mut display = {
        let rgb = true;
        let inverted = false;
        
        let rst = PinDriver::output(peripherals.pins.gpio10)?;
        let sda = peripherals.pins.gpio11;
        let sck = peripherals.pins.gpio12;
        let a0 = PinDriver::output(peripherals.pins.gpio13)?;
        
        let spi = SpiDeviceDriver::new_single(
            peripherals.spi2,
            sck,
            sda,
            AnyIOPin::none(),
            AnyIOPin::none(),
            &DriverConfig {
                dma: Dma::Auto(128 * 160 * 2),
                intr_flags: Default::default(),
            },
            &SpiConfig::new().baudrate(30.MHz().into())
        )?;
        
        ST7735::new(spi, a0, rst, rgb, inverted, SCR_WIDTH as u32, SCR_HEIGHT as u32)
    };
    
    display.init(&mut FreeRtos).map_err(|_| DisplayError::InitError)?;
    display.set_orientation(&Orientation::Landscape).map_err(|_| DisplayError::SetOrientationError)?;
    display.set_offset(1, 2); // No idea why its needed
    display.set_address_window(0, 0, 159, 127).map_err(|_| DisplayError::SetOrientationError)?;

    log::info!("Hello, world!");
    
    let mut framebuffer = vec![0; 128 * 160];
    
    loop {
        display.write_pixels_buffered(framebuffer.iter().copied()).map_err(|_| DisplayError::DrawError)?;
        
        FreeRtos::delay_ms(10);
        
        if start_btn.falling_edge() {
            log::info!("start");
            
            let start = Instant::now();
            let mut frames = 0;
            let mut parts = (0.0, 0.0, 0.0);
            let mut decoder = rle::Decoder::new(VIDEO);
            let mut row = [0; 160];
            
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
                        framebuffer[x + y * 160] = Color::new(color, color, color).into();
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
        
        framebuffer.fill(Color::WHITE.into());
        
        let mut draw_rect = |filled: bool, x: usize, y: usize, w: usize, h: usize, color: Color| {
            for row in y..(y + h) {
                if filled || (row == y) || (row == y + h - 1) {
                    framebuffer[row * SCR_WIDTH + x .. row * SCR_WIDTH + x + w].fill(color.into());
                } else {
                    framebuffer[row * SCR_WIDTH + x] = color.into();
                    framebuffer[row * SCR_WIDTH + x + w - 1] = color.into();
                }
            }
        };
        
        draw_rect(analog_btn.is_low(), 10, 15, 60, 60, Color::BLACK);
        draw_rect(true, (40 + analog_x.read(27)? - 2) as usize, (45 + analog_y.read(27)? - 2) as usize, 4, 4, if analog_btn.is_low() { Color::WHITE } else { Color::BLACK });
        
        draw_rect(x_btn.is_low(), 80, 10, 30, 30, Color::BLUE);
        draw_rect(y_btn.is_low(), 120, 10, 30, 30, Color::YELLOW);
        draw_rect(a_btn.is_low(), 80, 50, 30, 30, Color::GREEN);
        draw_rect(b_btn.is_low(), 120, 50, 30, 30, Color::RED);
        
        draw_rect(select_btn.is_low(), 20, 90, 40, 20, Color::TEAL);
        draw_rect(start_btn.is_low(), 100, 90, 40, 20, Color::MAGENTA);
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
