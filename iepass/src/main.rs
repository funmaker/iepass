#![feature(try_blocks)]
#![feature(iter_array_chunks)]

use std::time::Instant;
use iepass_core::rle;
use iepass_core::colors::Color;
use embedded_io::{Read, ReadExactError};
use esp_idf_svc::hal::adc::oneshot::AdcDriver;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::SpiDriver;

mod utils;
mod debounce;
mod analog;
mod touch;
mod calib;
mod display;
mod sound;

use utils::{draw_rect, SCR_HEIGHT, SCR_WIDTH};
use debounce::Debounce;
use analog::Analog;
use calib::Calib;
use touch::Touch;
use display::Display;
use sound::Sound;

// == Sound ==
//    LRC:  5
//   RCLK:  6
//    DIN:  7
// == Analog ==
//      X:  8
//      Y:  9
//    BTN:  3
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
static KUTASAN: &[u8] = include_bytes!("../../assets/kutasan.pcm");

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// It is necessary to call this function once. Otherwise, some patches to the runtime
	// implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
	esp_idf_svc::sys::link_patches();
	// Bind the log crate to the ESP Logging facilities
	esp_idf_svc::log::EspLogger::initialize_default();
	
	let calib = Calib::default();
	let peripherals = Peripherals::take().unwrap();
	
	let mut select_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio14)?).with_pull(Pull::Up)?;
	let mut start_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio4)?).with_pull(Pull::Up)?;
	let mut x_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio15)?).with_pull(Pull::Up)?;
	let mut y_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio16)?).with_pull(Pull::Up)?;
	let mut a_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio17)?).with_pull(Pull::Up)?;
	let mut b_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio18)?).with_pull(Pull::Up)?;
	
	let adc1_driver = AdcDriver::new(peripherals.adc1)?;
	let mut analog = Analog::new(
		&adc1_driver,
		peripherals.pins.gpio8,
		peripherals.pins.gpio9,
		calib.analog_deadzone,
		calib.analog,
	)?;
	let mut analog_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio3)?).with_pull(Pull::Up)?;
	
	let mut sd_cs = PinDriver::output(peripherals.pins.gpio38)?;
	sd_cs.set_high()?;
	
	let touch_sd_spi = SpiDriver::new(
		peripherals.spi3,
		peripherals.pins.gpio36,
		peripherals.pins.gpio35,
		Some(peripherals.pins.gpio37),
		&Default::default(),
	)?;
	
	let mut touch = Touch::new(
		&touch_sd_spi,
		Some(peripherals.pins.gpio40),
		peripherals.pins.gpio39,
		calib.touch,
	)?;
	
	let mut display = Display::new(
		peripherals.spi2,
		peripherals.pins.gpio12,
		peripherals.pins.gpio11,
		peripherals.pins.gpio13,
		peripherals.pins.gpio10,
		calib.screen_offset,
	)?;
	
	let mut sound = Sound::new(
		peripherals.i2s0,
		peripherals.pins.gpio6,
		peripherals.pins.gpio7,
		peripherals.pins.gpio5,
	)?;
	
	log::info!("Hello, world!");
	
	let mut framebuffer = vec![0; SCR_WIDTH as usize * SCR_HEIGHT as usize];
	
	loop {
		display.update(&framebuffer)?;
		
		FreeRtos::delay_ms(10);
		
		if select_btn.falling_edge() {
			log::info!("select");
			sound.play(&KUTASAN)?;
			log::info!("select done");
		}
		
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
				for y in 0..SCR_HEIGHT as usize {
					if start_btn.falling_edge() {
						break 'outer;
					}
					
					match decoder.read_exact(&mut row) {
						Err(ReadExactError::UnexpectedEof) => break 'outer,
						result => result?,
					}
					
					for x in 0..160 {
						let color = row[x];
						framebuffer[x + y * SCR_WIDTH as usize] = Color::new(color, color, color).into();
					}
				}
				
				parts.0 += now.elapsed().as_secs_f32();
				let now = Instant::now();
				
				display.update(&framebuffer)?;
				
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
		
		let (analog_x, analog_y) = analog.read(27)?;
		draw_rect(&mut framebuffer, analog_btn.is_low(), 10, 15, 60, 60, Color::BLACK);
		draw_rect(&mut framebuffer, true, (40 + analog_x - 2) as u16, (45 + analog_y - 2) as u16, 4, 4, if analog_btn.is_low() { Color::WHITE } else { Color::BLACK });
		
		draw_rect(&mut framebuffer, x_btn.is_low(), 80, 10, 30, 30, Color::BLUE);
		draw_rect(&mut framebuffer, y_btn.is_low(), 120, 10, 30, 30, Color::YELLOW);
		draw_rect(&mut framebuffer, a_btn.is_low(), 80, 50, 30, 30, Color::GREEN);
		draw_rect(&mut framebuffer, b_btn.is_low(), 120, 50, 30, 30, Color::RED);
		
		draw_rect(&mut framebuffer, select_btn.is_low(), 20, 90, 40, 20, Color::TEAL);
		draw_rect(&mut framebuffer, start_btn.is_low(), 100, 90, 40, 20, Color::MAGENTA);
		
		if let Some((x, y)) = touch.read(SCR_WIDTH, SCR_HEIGHT)? {
			draw_rect(&mut framebuffer, true, x.saturating_sub(4), y.saturating_sub(4), 8, 8, Color::RED);
		}
	}
}
