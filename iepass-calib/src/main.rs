#![no_std]
#![no_main]
#![deny(
	clippy::mem_forget,
	reason = "mem::forget is generally not safe to do with esp_hal types, especially those holding buffers for the duration of a data transfer."
)]

use panic_rtt_target as _;
use defmt::info;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embedded_hal_async::delay::DelayNs;
use embedded_io::Read;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{self, Input, Level, Output, Pull};
use esp_hal::timer::systimer::SystemTimer;
use iepass_core::colors::Color;
use iepass::peripherials::{Analog, Debounce, Display, Speaker, SpiBus, Touch};
use iepass::calib::Calib;
use iepass::static_framebuffer;

static CALIB_BG1: &[u8] = include_bytes!("../../assets/calib1.smol");
static CALIB_BG2: &[u8] = include_bytes!("../../assets/calib2.smol");
static CALIB_BG3: &[u8] = include_bytes!("../../assets/calib3.smol");
static CALIB_BG4: &[u8] = include_bytes!("../../assets/calib4.smol");
static CALIB_BG5: &[u8] = include_bytes!("../../assets/calib5.smol");

static KUTASAN: &[u8] = include_bytes!("../../assets/kutasan.pcm");

const SCR_WIDTH: u16 = 160;
const SCR_HEIGHT: u16 = 128;
const ANALOG_SIZE: i16 = 50;

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

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
	rtt_target::rtt_init_defmt!();
	
	try_main(spawner).await
	                 .expect("Error in the main task");
}

async fn try_main(_spawner: Spawner) -> Result<()> {
	info!("Initializing system.");
	
	let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
	let mut calib = Calib::default();
	let peripherals = esp_hal::init(config);
	
	info!("Initializing allocators.");
	
	esp_alloc::heap_allocator!(size: 64 * 1024);
	
	info!("Initializing embassy.");
	
	let timer0 = SystemTimer::new(peripherals.SYSTIMER);
	esp_hal_embassy::init(timer0.alarm0);
	
	info!("Initializing peripherals.");
	
	let up_pull = gpio::InputConfig::default().with_pull(Pull::Up);
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
	
	let mut display = Display::new(
		peripherals.SPI2,
		peripherals.DMA_CH0,
		peripherals.GPIO12,
		peripherals.GPIO11,
		peripherals.GPIO13,
		peripherals.GPIO10,
		calib.screen_offset,
		embassy_time::Delay,
	).await?;
	
	let mut framebuffer = static_framebuffer!();
	
	framebuffer.fill_iter(pixels(CALIB_BG1));
	calib.screen_offset.x = 0;
	calib.screen_offset.y = 0;
	
	info!("Calibrating screen offset. Use buttons to change offset.");
	info!("Press START to accept.");
	loop {
		embassy_time::Delay.delay_ms(10).await;
	
		if a_btn.falling_edge() { calib.screen_offset.y = calib.screen_offset.y.saturating_add(1); }
		if b_btn.falling_edge() { calib.screen_offset.x = calib.screen_offset.x.saturating_add(1); }
		if x_btn.falling_edge() { calib.screen_offset.x = calib.screen_offset.x.saturating_sub(1); }
		if y_btn.falling_edge() { calib.screen_offset.y = calib.screen_offset.y.saturating_sub(1); }
	
		display.apply_calib(calib.screen_offset)?;
		display.draw_async(&mut framebuffer).await?;
	
		if start_btn.falling_edge() {
			info!("Screen offset: x = {}, y = {}", calib.screen_offset.x, calib.screen_offset.y);
			break;
		}
	}
	
	info!("Calibrating analog center. Please wait...");
	framebuffer.fill_iter(pixels(CALIB_BG2));
	display.draw_async(&mut framebuffer).await?;
	embassy_time::Delay.delay_ms(1000).await;
	
	let center = measure(64, || analog.read_raw());
	
	calib.analog_deadzone = 6;
	calib.analog.x.min = center.0.saturating_sub(calib.analog_deadzone);
	calib.analog.x.mid = center.0;
	calib.analog.x.max = center.0.saturating_add(calib.analog_deadzone);
	calib.analog.y.min = center.1.saturating_sub(calib.analog_deadzone);
	calib.analog.y.mid = center.1;
	calib.analog.y.max = center.1.saturating_add(calib.analog_deadzone);
	
	analog.deadzone = calib.analog_deadzone;
	analog.calib = calib.analog;
	
	info!("Analog center: x = {}, y = {}", calib.analog.x.mid, calib.analog.y.mid);
	
	info!("Calibrating analog range. Move the stick to all the corners.");
	info!("Press SELECT to reset.");
	info!("Press START to accept.");
	loop {
		embassy_time::Delay.delay_ms(10).await;
		
		let (x, y) = analog.read_raw();
		
		calib.analog.x.min = calib.analog.x.min.min(x.saturating_add(analog.deadzone / 2));
		calib.analog.x.max = calib.analog.x.max.max(x.saturating_sub(analog.deadzone / 2));
		calib.analog.y.min = calib.analog.y.min.min(y.saturating_add(analog.deadzone / 2));
		calib.analog.y.max = calib.analog.y.max.max(y.saturating_sub(analog.deadzone / 2));
		analog.calib = calib.analog;
		
		let (x, y) = analog.rescale(x, y, ANALOG_SIZE - 2);
		
		framebuffer.fill_iter(pixels(CALIB_BG3));
		framebuffer.draw_rect(
			true,
			(SCR_WIDTH / 2).saturating_add_signed(x).saturating_sub(2),
			(SCR_HEIGHT / 2).saturating_add_signed(y).saturating_sub(2),
			4,
			4,
			Color::RED
		);
		display.draw_async(&mut framebuffer).await?;
		
		if select_btn.falling_edge() {
			calib.analog.x.min = calib.analog.x.mid.saturating_sub(calib.analog_deadzone);
			calib.analog.x.max = calib.analog.x.mid.saturating_add(calib.analog_deadzone);
			calib.analog.y.min = calib.analog.y.mid.saturating_sub(calib.analog_deadzone);
			calib.analog.y.max = calib.analog.y.mid.saturating_add(calib.analog_deadzone);
		}
	
		if start_btn.falling_edge() {
			info!("X axis: min = {}, mid = {}, max = {}", calib.analog.x.min, calib.analog.x.mid, calib.analog.x.max);
			info!("Y axis: min = {}, mid = {}, max = {}", calib.analog.y.min, calib.analog.y.mid, calib.analog.y.max);
			break;
		}
	}
	
	let mut measurements = [
		((0, 0), (32, 32)),
		((0, 0), (SCR_WIDTH - 32, 32)),
		((0, 0), (SCR_WIDTH - 32, SCR_HEIGHT - 32)),
		((0, 0), (32, SCR_HEIGHT - 32)),
	];
	
	'outer: loop {
		info!("Calibrating touch screen. Press targets using pen.");
		info!("Press SELECT to reset.");
		
		for &mut (ref mut measurement, (x, y)) in measurements.iter_mut() {
			framebuffer.fill_iter(pixels(CALIB_BG4));
			framebuffer.draw_rect(true, x - 4, y - 4, 8, 8, Color::BLACK);
			display.draw_async(&mut framebuffer).await?;
			
			loop {
				embassy_time::Delay.delay_ms(10).await;
				
				if let Some(value) = measure_option(64, || block_on(touch.read_raw()))? {
					*measurement = value;
					break;
				}
				
				if select_btn.falling_edge() {
					continue 'outer;
				}
			}
			
			while let Some(_) = touch.read_raw().await? {
				embassy_time::Delay.delay_ms(10).await;
			}
		}
		
		[calib.touch.x.min, calib.touch.x.max] = fit_map(measurements.iter().map(|(_, (x, _))| *x), measurements.iter().map(|((x, _), _)| *x), [0, SCR_WIDTH]);
		[calib.touch.y.min, calib.touch.y.max] = fit_map(measurements.iter().map(|(_, (_, y))| *y), measurements.iter().map(|((_, y), _)| *y), [0, SCR_HEIGHT]);
		
		touch.apply_calib(calib.touch);
		
		info!("Press START to accept.");
		
		loop {
			embassy_time::Delay.delay_ms(10).await;
			
			framebuffer.fill_iter(pixels(CALIB_BG4));
			
			if let Some((x, y)) = measure_option(16, || block_on(touch.read(SCR_WIDTH, SCR_HEIGHT)))? {
				framebuffer.draw_rect(true, x.saturating_sub(4), y.saturating_sub(4), 8, 8, Color::RED);
			}
			
			display.draw_async(&mut framebuffer).await?;
			
			if select_btn.falling_edge() {
				continue 'outer;
			}
			
			if start_btn.falling_edge() {
				break 'outer;
			}
		}
	}
	
	info!("Generated calibration config:");
	info!("{:#?}", calib);
	
	framebuffer.fill_iter(pixels(CALIB_BG5));
	display.draw_async(&mut framebuffer).await?;
	speaker.play(&*KUTASAN).await?;
	speaker.reset().await?;
	
	loop {
		embassy_time::Delay.delay_ms(1000).await;
	}
}

fn pixels(smol: &[u8]) -> impl Iterator<Item = Color> {
	let mut decoder = iepass_core::rle::Decoder::new(smol);
	
	core::iter::from_fn(move || {
		let mut value = [0, 0];
		decoder.read_exact(&mut value)
		       .ok()
		       .map(|_| Color::from_raw(u16::from_be_bytes(value)))
	})
}

fn measure(count: u32, mut func: impl FnMut() -> (u16, u16)) -> (u16, u16) {
	let mut x = 0;
	let mut y = 0;
	
	for _ in 0..count {
		let (vx, vy) = func();
		x += vx as u32;
		y += vy as u32;
	}
	
	((x / count) as u16, (y / count) as u16)
}

fn measure_option<E>(count: u32, mut func: impl FnMut() -> Result<Option<(u16, u16)>, E>) -> Result<Option<(u16, u16)>, E> {
	let mut x = 0;
	let mut y = 0;
	
	for _ in 0..count {
		match func()? {
			Some((vx, vy)) => {
				x += vx as u32;
				y += vy as u32;
			},
			None => return Ok(None),
		}
	}
	
	Ok(Some(((x / count) as u16, (y / count) as u16)))
}

fn fit_map<const N: usize>(x: impl Iterator<Item = u16> + Clone,
                           y: impl Iterator<Item = u16> + Clone,
                           points: [u16; N])
                           -> [u16; N] {
	let mean_x = x.clone().fold(0.0, |a, b| a + b as f32) / x.clone().count() as f32;
	let mean_y = y.clone().fold(0.0, |a, b| a + b as f32) / y.clone().count() as f32;
	
	let mut covxy = 0.0;
	let mut varx = 0.0;
	
	for (x, y) in x.zip(y) {
		let dx = x as f32 - mean_x;
		covxy += dx * (y as f32 - mean_y);
		varx  += dx * dx;
	}
	
	assert!(varx.abs() > 1e-12);
	
	let a = covxy / varx;        // slope
	let b = mean_y - a * mean_x; // intercept
	
	points.map(|x| (x as f32 * a + b) as u16)
}
