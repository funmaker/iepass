#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(arc_is_unique)]

use std::collections::HashSet;
use std::fs;
use std::ops::{Sub};
use std::time::{Duration, Instant};
use eframe::{egui, CreationContext};
use eframe::epaint::TextureHandle;
use egui::{Color32, Event, Frame, ImageSource, Key, RawInput};
use egui::load::SizedTexture;
use iepass_core::pico8::{Pico8VM, RunResult};
use iepass_core::colors::Color;
use iepass_core::pico8::palette::PALETTE;

mod framebuffer_pool;

use framebuffer_pool::{FramebufferPool, FRAMEBUFFER_OPTS};

fn main() -> eframe::Result {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 512.0]),
		..Default::default()
	};
	
	let mut args = std::env::args().rev().collect::<Vec<_>>();
	args.pop();
	let cart = args.pop();
	
	if !args.is_empty() {
		eprintln!("Unexpected argument: {}", args.pop().unwrap());
	}
	
	eframe::run_native(
		"IEPass Emulator",
		options,
		Box::new(|cc| {
			Ok(Box::new(EmulatorApp::new(cc, cart)))
		}),
	)
}

struct EmulatorApp {
	fb_pool: FramebufferPool,
	fb_tex: TextureHandle,
	frame: usize,
	pressed_keys: HashSet<Key>,
	last_frames: [Instant; 10],
	target_fps: u16,
	pico8: Pico8VM,
	running: bool,
}

impl EmulatorApp {
	pub fn new(cc: &CreationContext, cart_path: Option<String>) -> EmulatorApp {
		let mut fb_pool = FramebufferPool::new(128, 128);
		let fb_tex = cc.egui_ctx.load_texture("framebuffer", fb_pool.from_color(Color32::MAGENTA), FRAMEBUFFER_OPTS);
		
		let mut pico8 = Pico8VM::new().unwrap();
		
		let load_result = if let Some(cart_path) = cart_path {
			match fs::read_to_string(&cart_path) {
				Ok(cart) => pico8.load_cartridge(cart),
				Err(err) => {
					eprintln!("Failed to open cartridge: {}", err);
					Ok(())
				},
			}
		} else {
			pico8.load_cartridge(include_bytes!("../../lua/hello.p8"))
		};
		
		match load_result {
			Ok(_) => eprintln!("Successfully loaded cartridge."),
			Err(err) => eprintln!("Failed to load cartridge: {}", err),
		}
		
		Self {
			fb_pool,
			fb_tex,
			frame: 0,
			last_frames: [Instant::now().sub(Duration::from_millis(1000)); 10],
			target_fps: 30,
			pico8,
			running: true,
			pressed_keys: HashSet::new(),
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
			{
				let rt = self.pico8.runtime();
				
				let mut buttons = [0u8; 8];
				let p1_buttons = &mut buttons[0];
				if self.pressed_keys.contains(&Key::ArrowUp) { *p1_buttons |= 0x4 }
				if self.pressed_keys.contains(&Key::ArrowDown) { *p1_buttons |= 0x8 }
				
				rt.update_buttons(&buttons);
				rt.finish_update_frame();
			}
			
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
				
				let screen_palette = rt.memory.palette(1);
				
				let map_color = |color: u8| -> Color {
					assert!(color < 16);
					PALETTE[(screen_palette[color as usize] as usize) & 0x0F]
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
					if *pressed {
						self.pressed_keys.insert(*key);
					} else {
						self.pressed_keys.remove(key);
					}
				},
				_ => {},
			}
		}
	}
}
