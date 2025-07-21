use std::io::{Read, Write};
use embedded_graphics_core::pixelcolor::raw::RawU16;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::primitives::Rectangle;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Gpio0, PinDriver, Pull};
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver};
use st7735_lcd::{Orientation, ST7735};
use thiserror::Error;

mod debounce;
mod rle;

use crate::debounce::Debounce;

static BAD_APPLE: &[u8] = include_bytes!("../assets/BadApple.smol");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
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
        let rgb = false;
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
            &Default::default(),
            &SpiConfig::new().baudrate(30.MHz().into())
        )?;
        
        ST7735::new(spi, a0, rst, rgb, inverted, width, height)
    };
    
    display.init(&mut FreeRtos).map_err(|_| DisplayError::InitError)?;
    display.set_orientation(&Orientation::Landscape).map_err(|_| DisplayError::SetOrientationError)?;
    display.clear(Rgb565::MAGENTA).map_err(|_| DisplayError::ClearError)?;
    // display.fill_contiguous(
    //     &Rectangle::new(Point::new(32, 32), Size::new(94, 64)),
    //     (0_u32..).map(|n| {
    //         let x = n % 94;
    //         let y = n / 94;
    //
    //         Rgb565::new(
    //             (x * (1 << 5) / 94) as u8,
    //             0,
    //             (y * (1 << 5) / 64) as u8,
    //         )
    //     }),
    // ).map_err(|_| DisplayError::FillContinuousError)?;

    log::info!("Hello, world!");
    
    let mut video_reader = None;
    let mut frame = Vec::with_capacity(160 * 128);
    frame.resize(160 * 128, 0_u8);
    
    loop {
        FreeRtos::delay_ms(10);
        
        if select_btn.falling_edge() {
            log::info!("select");
            display.clear(Rgb565::MAGENTA).map_err(|_| DisplayError::ClearError)?;
            display.fill_solid(
                &Rectangle::new(Point::new(0, 0), Size::new(160, 128)),
                Rgb565::MAGENTA,
            ).map_err(|_| DisplayError::FillError)?;
            video_reader = None;
        }
        if start_btn.falling_edge() {
            log::info!("start");
            video_reader = Some(rle::Decoder::new(BAD_APPLE));
            display.set_offset(0, 0);
            display.set_address_window(0, 0, 159, 127).map_err(|_| DisplayError::SetOrientationError)?;
            log::info!("start2");
        }
        if a_btn.falling_edge() {
            log::info!("a");
            display.fill_solid(
                &Rectangle::new(Point::new(16, 128 - 48), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::FillError)?;
        }
        if b_btn.falling_edge() {
            log::info!("b");
            display.fill_solid(
                &Rectangle::new(Point::new(160 - 48, 128 - 48), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::FillError)?;
        }
        if x_btn.falling_edge() {
            log::info!("x");
            display.fill_solid(
                &Rectangle::new(Point::new(16, 16), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::FillError)?;
        }
        if y_btn.falling_edge() {
            log::info!("y");
            display.fill_solid(
                &Rectangle::new(Point::new(160 - 48, 16), Size::new(32, 32)),
                Rgb565::BLUE,
            ).map_err(|_| DisplayError::FillError)?;
        }
        
        if let Some(video) = video_reader.as_mut() {
            match video.read_exact(&mut frame) {
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    video_reader = None;
                    continue;
                },
                result => result?,
            }
        
            display.write_pixels_buffered(
                frame.iter()
                     .copied()
                     .map(|byte| RawU16::from(Rgb565::new(
                         ((byte as u16) * (1 << 5) / 256) as u8,
                         ((byte as u16) * (1 << 6) / 256) as u8,
                         ((byte as u16) * (1 << 5) / 256) as u8,
                     )).into_inner()),
            ).map_err(|_| DisplayError::FillError)?;
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
    FillError,
}
