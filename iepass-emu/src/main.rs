#![feature(arc_is_unique)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ops::{Sub};
use std::time::{Duration, Instant};
use eframe::{egui, CreationContext};
use eframe::epaint::TextureHandle;
use egui::{Color32, Event, Frame, ImageSource, RawInput};
use egui::load::SizedTexture;
use iepass_core::pico8::Pico8VM;
use iepass_core::colors::Color;
use iepass_core::pico8::palette::PALETTE;

mod framebuffer_pool;

use framebuffer_pool::{FramebufferPool, FRAMEBUFFER_OPTS};

fn main() -> eframe::Result {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 512.0]),
		..Default::default()
	};
	
	eframe::run_native(
		"IEPass Emulator",
		options,
		Box::new(|cc| {
			Ok(Box::new(EmulatorApp::new(cc)))
		}),
	)
}

struct EmulatorApp {
	fb_pool: FramebufferPool,
	fb_tex: TextureHandle,
	frame: usize,
	last_frames: [Instant; 10],
	requested_fps: u16,
	pico8: Pico8VM,
	running: bool,
}

impl EmulatorApp {
	pub fn new(cc: &CreationContext) -> EmulatorApp {
		let mut fb_pool = FramebufferPool::new(128, 128);
		let fb_tex = cc.egui_ctx.load_texture("framebuffer", fb_pool.from_color(Color32::MAGENTA), FRAMEBUFFER_OPTS);
		
		
		let mut pico8 = Pico8VM::new().unwrap();
		pico8.load(include_bytes!("../../lua/hello.lua"));
		
		Self {
			fb_pool,
			fb_tex,
			frame: 0,
			last_frames: [Instant::now().sub(Duration::from_millis(1000)); 10],
			requested_fps: 30,
			pico8,
			running: true,
		}
	}
}

impl eframe::App for EmulatorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let now = Instant::now();
		
		let elapsed = now - self.last_frames[0];
		let previous_duration = self.last_frames[0] - self.last_frames[2];
		
		let requested_delay = 1f32 / self.requested_fps as f32;
		let previous_error = previous_duration.as_secs_f32()/2f32 - requested_delay;
		
		let delta = requested_delay - elapsed.as_secs_f32() - 0.5f32*previous_error.clamp(-requested_delay*0.9f32, requested_delay*0.9f32);
		
		if delta < 0.001f32 && self.running {
			let mut run_result = self.pico8.run_fuel(25000);
			while run_result.out_of_fuel && (Instant::now() - now).as_secs_f32() < requested_delay {
				run_result = self.pico8.run_fuel(25000);
			}
			
			self.requested_fps = if run_result.stopped { 10 } else { run_result.requested_fps.max(1) };
			// println!("Target FPS {}, since last frame: {:.1} ({:.1} fps)", self.requested_fps, elapsed.as_secs_f32() * 1000f32, 1f32/elapsed.as_secs_f32());
			
			if run_result.stopped {
				self.running = false;
			}
			
			if !run_result.out_of_fuel {
				let mut env = self.pico8.env();
				
				let screen_palette = env.memory.palette(1);
				
				let map_color = |color: u8| -> Color {
					assert!(color < 16);
					PALETTE[(screen_palette[color as usize] as usize) & 0x0F]
				};
				
				self.fb_tex.set(self.fb_pool.from_iter(
					env
						.memory
						.screen()
						.iter()
						.map(|byte| [map_color(*byte >> 4), map_color(*byte & 0x0F)])
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
					modifiers,
					physical_key,
					repeat,
					..
				} => {
					println!("{} {:?} {:?} {:?} {}", pressed, key, modifiers, physical_key, repeat);
				},
				_ => {},
			}
		}
	}
}
