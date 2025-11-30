#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(arc_is_unique)]

#[macro_use] extern crate p8rs;

use std::cell::Cell;
use std::fs;
use std::ops::{Not, Sub};
use std::rc::Rc;
use std::time::{Duration, Instant};
use eframe::{egui, CreationContext};
use eframe::epaint::TextureHandle;
use egui::{Color32, Event, Frame, ImageSource, Key, RawInput};
use egui::load::SizedTexture;
use p8rs::vm::{palette, Callbacks, P8rs, RunResult};
use p8rs::colors::Color;

mod framebuffer_pool;

use framebuffer_pool::{FramebufferPool, FRAMEBUFFER_OPTS};
use p8rs::vm::memory::machine_state::Palette;
use p8rs_types::p8scii;

fn main() -> eframe::Result {
	env_logger::init();
	
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 512.0]),
		..Default::default()
	};
	
	let mut args = std::env::args().rev().collect::<Vec<_>>();
	args.pop();
	let cart = args.pop();
	
	if !args.is_empty() {
		error!("Unexpected argument: {}", args.pop().unwrap());
	}
	
	eframe::run_native(
		"IEPass Emulator",
		options,
		Box::new(|cc| {
			Ok(Box::new(EmulatorApp::new(cc, cart)))
		}),
	)
}

struct EmulatorCallbacks {
	buttons: Rc<Cell<[u8; 8]>>,
}

impl Callbacks for EmulatorCallbacks {
	fn printh(&mut self, text: &[u8], _filename: Option<&[u8]>, _overwrite: Option<bool>, _save_to_desktop: Option<bool>) {
		println!("INFO: {}", p8scii::Display(text));
	}
	
	fn get_buttons(&mut self) -> [u8; 8] {
		self.buttons.get()
	}
}

struct EmulatorApp {
	fb_pool: FramebufferPool,
	fb_tex: TextureHandle,
	frame: usize,
	pressed_buttons: Rc<Cell<[u8; 8]>>,
	last_frames: [Instant; 10],
	target_fps: u16,
	pico8: P8rs,
	running: bool,
}

impl EmulatorApp {
	pub fn new(cc: &CreationContext, cart_path: Option<String>) -> EmulatorApp {
		let mut fb_pool = FramebufferPool::new(128, 128);
		let fb_tex = cc.egui_ctx.load_texture("framebuffer", fb_pool.from_color(Color32::MAGENTA), FRAMEBUFFER_OPTS);
		
		let mut pico8 = P8rs::new().unwrap();
		
		let load_result = if let Some(cart_path) = cart_path {
			match fs::read_to_string(&cart_path) {
				Ok(cart) => pico8.load_cartridge(cart),
				Err(err) => {
					error!("Failed to open cartridge: {}", err);
					Ok(())
				},
			}
		} else {
			pico8.load_cartridge(include_bytes!("../../lua/hello.p8"))
		};
		
		match load_result {
			Ok(_) => info!("Successfully loaded cartridge."),
			Err(err) => error!("Failed to load cartridge: {}", err),
		}
		
		let pressed_buttons = Rc::new(Cell::new([0; 8]));
		
		pico8.set_callbacks(EmulatorCallbacks {
			buttons: pressed_buttons.clone(),
		});
		
		Self {
			fb_pool,
			fb_tex,
			frame: 0,
			last_frames: [Instant::now().sub(Duration::from_millis(1000)); 10],
			target_fps: 30,
			pico8,
			running: true,
			pressed_buttons: pressed_buttons.clone(),
		}
	}
}

impl eframe::App for EmulatorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let now = Instant::now();
		
		let elapsed = now - self.last_frames[0];
		let previous_duration = self.last_frames[0] - self.last_frames[2];
		
		let requested_delay = 1f32 / self.target_fps as f32;
		let previous_error = previous_duration.as_secs_f32()/2f32 - requested_delay;
		
		let delta = requested_delay - elapsed.as_secs_f32() - 0.5f32*previous_error.clamp(-requested_delay*0.9f32, requested_delay*0.9f32);
		
		if delta < 0.001f32 && self.running {
			// {
			// 	let rt = self.pico8.runtime();
			//
			// 	let mut buttons = [0u8; 8];
			// 	let p1_buttons = &mut buttons[0];
			// 	let pressed_keys = self.pressed_buttons.();
			// 	if pressed_keys.contains(&Key::ArrowUp) { *p1_buttons |= 0x4 }
			// 	if pressed_keys.contains(&Key::ArrowDown) { *p1_buttons |= 0x8 }
			//
			// 	rt.update_buttons(&buttons);
			// 	rt.finish_update_frame();
			// }
			
			let mut run_result = self.pico8.run_fuel(25000).unwrap();
			while run_result == RunResult::OutOfFuel && (Instant::now() - now).as_secs_f32() < requested_delay {
				run_result = self.pico8.run_fuel(25000).unwrap();
			}
			
			self.target_fps = if run_result == RunResult::Stop { 10 } else { self.pico8.runtime().target_fps.max(1) };
			
			if run_result == RunResult::Stop {
				self.running = false;
			}
			
			if run_result != RunResult::OutOfFuel {
				let rt = self.pico8.runtime();
				
				let screen_palette = *rt.memory.machine_state().palette(Palette::Screen);
				
				let map_color = |color: u8| -> Color {
					assert!(color < 16);
					palette::color_from_index(screen_palette[color as usize])
				};
				
				self.fb_tex.set(self.fb_pool.from_iter(
					rt
						.memory
						.screen()
						.iter()
						.map(|byte| [map_color(*byte & 0x0F), map_color(*byte >> 4)])
						.flatten()
						.map(|color| {
							let (r, g, b) = color.rgb();
							Color32::from_rgb(r, g, b)
						})
				), FRAMEBUFFER_OPTS);
				
				self.frame = self.frame + 1;
				self.last_frames.rotate_right(1);
				self.last_frames[0] = now;
			}
		}
		
		egui::SidePanel::left("framebuffer")
			.frame(Frame::NONE)
			.exact_width(ctx.available_rect().height())
			.resizable(false)
			.show_separator_line(false)
			.show(ctx, |ui| {
				egui::Image::new(ImageSource::Texture(SizedTexture::new(self.fb_tex.id(), [128.0, 128.0])))
					.paint_at(ui, ui.max_rect());
			});
		
		egui::CentralPanel::default()
			.show(ctx, |ui| {
				ui.heading("IE Pass: The Console The Pass The Emulator");
				ui.separator();
				ui.label(format!("{}", if self.running { "Running" } else { "Stopped" }));
				ui.label(format!("Frame {}", self.frame));
				ui.label(format!("FPS: {:>4.0}", self.last_frames.len() as f32 / self.last_frames.last().unwrap().elapsed().as_secs_f32()));
			});
		
		ctx.request_repaint();
	}
	
	fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut RawInput) {
		for event in &raw_input.events {
			match event {
				Event::Key {
					pressed,
					key,
					..
				} => {
					let player_idx = 0;
					let btn_idx = match *key {
						Key::ArrowLeft => Some(0u8),
						Key::ArrowRight => Some(1),
						Key::ArrowUp => Some(2),
						Key::ArrowDown => Some(3),
						Key::C => Some(4),
						Key::N => Some(4),
						Key::Z => Some(4),
						Key::M => Some(5),
						Key::V => Some(5),
						Key::X => Some(5),
						_ => None,
					};
						if let Some(btn_idx) = btn_idx {
							let mut buttons = self.pressed_buttons.get();
							if *pressed {
								buttons[player_idx] = buttons[player_idx] | (1u8 << btn_idx);
							} else {
								buttons[player_idx] = buttons[player_idx] & (1u8 << btn_idx).not();
							}
							self.pressed_buttons.set(buttons);
						}
				},
				_ => {},
			}
		}
	}
}
