use embedded_io::Read;
use iepass_core::rle;
use thiserror::Error;
use esp_idf_svc::hal::adc::oneshot::AdcDriver;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::SpiDriver;
use iepass::{Debounce, Touch, Analog, Calib, Color};
use iepass::display::Display;
use iepass::utils::draw_rect;

static CALIB_BG1: &[u8] = include_bytes!("../../assets/calib1.smol");
static CALIB_BG2: &[u8] = include_bytes!("../../assets/calib2.smol");
static CALIB_BG3: &[u8] = include_bytes!("../../assets/calib3.smol");
static CALIB_BG4: &[u8] = include_bytes!("../../assets/calib4.smol");
static CALIB_BG5: &[u8] = include_bytes!("../../assets/calib5.smol");

const SCR_WIDTH: u16 = 160;
const SCR_HEIGHT: u16 = 128;
const ANALOG_SIZE: i16 = 50;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    let mut calib = Calib::default();
    let peripherals = Peripherals::take().unwrap();
    
    let mut select_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio14)?).with_pull(Pull::Up)?;
    let mut start_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio4)?).with_pull(Pull::Up)?;
    let mut x_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio15)?).with_pull(Pull::Up)?;
    let mut y_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio16)?).with_pull(Pull::Up)?;
    let mut a_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio17)?).with_pull(Pull::Up)?;
    let mut b_btn = Debounce::new(PinDriver::input(peripherals.pins.gpio18)?).with_pull(Pull::Up) ?;
    
    let analog_adc = AdcDriver::new(peripherals.adc2)?;
    let mut analog = Analog::new(
        &analog_adc,
        peripherals.pins.gpio19,
        peripherals.pins.gpio20,
        calib.analog_deadzone,
        calib.analog,
    )?;
    
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
    
    let mut framebuffer = vec![Color::WHITE.into(), SCR_WIDTH * SCR_HEIGHT];
    
    framebuffer.clear();
    framebuffer.extend(pixels(CALIB_BG1));
    
    log::info!("Calibrating screen offset. Use buttons to change offset.");
    log::info!("Press START to accept.");
    loop {
        FreeRtos::delay_ms(10);
    
        if a_btn.falling_edge() { calib.screen_offset.y = calib.screen_offset.y.saturating_add(1); }
        if b_btn.falling_edge() { calib.screen_offset.x = calib.screen_offset.x.saturating_add(1); }
        if x_btn.falling_edge() { calib.screen_offset.x = calib.screen_offset.x.saturating_sub(1); }
        if y_btn.falling_edge() { calib.screen_offset.y = calib.screen_offset.y.saturating_sub(1); }
    
        display.set_calib(calib.screen_offset)?;
	    display.update(&framebuffer)?;
    
        if start_btn.falling_edge() {
            log::info!("Screen offset: x = {}, y = {}", calib.screen_offset.x, calib.screen_offset.y);
            break;
        }
    }
    
    log::info!("Calibrating analog center. Please wait...");
    framebuffer.clear();
    framebuffer.extend(pixels(CALIB_BG2));
	display.update(&framebuffer)?;
    FreeRtos::delay_ms(1000);
    
    let center = measure(64, || analog.read_raw())?;
    
    calib.analog_deadzone = 6;
    calib.analog.x.min = center.0.saturating_sub(calib.analog_deadzone);
    calib.analog.x.mid = center.0;
    calib.analog.x.max = center.0.saturating_add(calib.analog_deadzone);
    calib.analog.y.min = center.1.saturating_sub(calib.analog_deadzone);
    calib.analog.y.mid = center.1;
    calib.analog.y.max = center.1.saturating_add(calib.analog_deadzone);
    
    analog.deadzone = calib.analog_deadzone;
	analog.calib = calib.analog;
    
    log::info!("Analog center: x = {}, y = {}", calib.analog.x.mid, calib.analog.y.mid);
    
    log::info!("Calibrating analog range. Move the stick to all the corners.");
    log::info!("Press SELECT to reset.");
    log::info!("Press START to accept.");
    loop {
	    FreeRtos::delay_ms(10);
	
	    let (x, y) = analog.read_raw()?;
	    
	    calib.analog.x.min = calib.analog.x.min.min(x.saturating_add(analog.deadzone / 2));
	    calib.analog.x.max = calib.analog.x.max.max(x.saturating_sub(analog.deadzone / 2));
	    calib.analog.y.min = calib.analog.y.min.min(y.saturating_add(analog.deadzone / 2));
	    calib.analog.y.max = calib.analog.y.max.max(y.saturating_sub(analog.deadzone / 2));
	    analog.calib = calib.analog;
	
	    let (x, y) = analog.rescale(x, y, ANALOG_SIZE - 2);
	
	    framebuffer.clear();
	    framebuffer.extend(pixels(CALIB_BG3));
	    draw_rect(
		    &mut framebuffer,
		    true,
		    (SCR_WIDTH / 2).saturating_add_signed(x).saturating_sub(2),
		    (SCR_HEIGHT / 2).saturating_add_signed(y).saturating_sub(2),
		    4,
		    4,
		    Color::RED
	    );
	    display.update(&framebuffer)?;
	
	    if select_btn.falling_edge() {
		    calib.analog.x.min = calib.analog.x.mid.saturating_sub(calib.analog_deadzone);
		    calib.analog.x.max = calib.analog.x.mid.saturating_add(calib.analog_deadzone);
		    calib.analog.y.min = calib.analog.y.mid.saturating_sub(calib.analog_deadzone);
		    calib.analog.y.max = calib.analog.y.mid.saturating_add(calib.analog_deadzone);
	    }
	
	    if start_btn.falling_edge() {
		    log::info!("X axis: min = {}, mid = {}, max = {}", calib.analog.x.min, calib.analog.x.mid, calib.analog.x.max);
		    log::info!("Y axis: min = {}, mid = {}, max = {}", calib.analog.y.min, calib.analog.y.mid, calib.analog.y.max);
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
	    log::info!("Calibrating touch screen. Press targets using pen.");
	    log::info!("Press SELECT to reset.");
	    
	    for &mut (ref mut measurement, (x, y)) in measurements.iter_mut() {
		    framebuffer.clear();
		    framebuffer.extend(pixels(CALIB_BG4));
		    draw_rect(&mut framebuffer, true, x - 4, y - 4, 8, 8, Color::BLACK);
		    display.update(&framebuffer)?;
		    
		    loop {
			    FreeRtos::delay_ms(10);
			    
			    if let Some(value) = measure_option(64, || touch.read_raw())? {
				    *measurement = value;
				    break;
			    }
			    
			    if select_btn.falling_edge() {
				    continue 'outer;
			    }
		    }
		    
		    while let Some(_) = touch.read_raw()? {
			    FreeRtos::delay_ms(10);
		    }
	    }
	    
	    
	    [calib.touch.x.min, calib.touch.x.max] = fit_map(measurements.iter().map(|(_, (x, _))| *x), measurements.iter().map(|((x, _), _)| *x), [0, SCR_WIDTH]);
	    [calib.touch.y.min, calib.touch.y.max] = fit_map(measurements.iter().map(|(_, (_, y))| *y), measurements.iter().map(|((_, y), _)| *y), [0, SCR_HEIGHT]);
	    
	    touch.calib = calib.touch;
	    
	    log::info!("{:?}", measurements);
	    log::info!("{:?}", [calib.touch.x.min, calib.touch.x.max]);
	    log::info!("{:?}", [calib.touch.y.min, calib.touch.y.max]);
	    log::info!("Press START to accept.");
	    
	    loop {
		    FreeRtos::delay_ms(10);
		    
		    framebuffer.clear();
		    framebuffer.extend(pixels(CALIB_BG4));
		    
		    if let Some((x, y)) = measure_option(16, || touch.read(SCR_WIDTH, SCR_HEIGHT))? {
			    draw_rect(&mut framebuffer, true, x.saturating_sub(4), y.saturating_sub(4), 8, 8, Color::RED);
		    }
		    
		    display.update(&framebuffer)?;
		    
		    if select_btn.falling_edge() {
			    continue 'outer;
		    }
		    
		    if start_btn.falling_edge() {
			    break 'outer;
		    }
	    }
    }
	
	log::info!("Generated calibration config:");
	log::info!("{calib:#?}");
	
	framebuffer.clear();
	framebuffer.extend(pixels(CALIB_BG5));
	display.update(&framebuffer)?;
    
    loop {
        FreeRtos::delay_ms(10);
    }
}

fn pixels(smol: &[u8]) -> impl Iterator<Item = u16> {
    let mut decoder = rle::Decoder::new(smol);
    
    std::iter::from_fn(move || {
        let mut value = [0, 0];
        decoder.read_exact(&mut value)
               .ok()
               .map(|_| u16::from_le_bytes(value))
    })
}

fn measure<E>(count: u32, mut func: impl FnMut() -> Result<(u16, u16), E>) -> Result<(u16, u16), E> {
	let mut x = 0;
	let mut y = 0;
	
	for _ in 0..count {
		let (vx, vy) = func()?;
		x += vx as u32;
		y += vy as u32;
	}
	
	Ok(((x / count) as u16, (y / count) as u16))
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
