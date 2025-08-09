#![feature(arc_is_unique)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;
use eframe::{egui, CreationContext};
use eframe::epaint::TextureHandle;
use egui::{Color32, Event, Frame, ImageSource, RawInput};
use egui::load::SizedTexture;
use iepass_core::pico8::Pico8VM;

mod framebuffer_pool;

use framebuffer_pool::{FramebufferPool, FRAMEBUFFER_OPTS};
use iepass_core::pico8::palette::PALETTE;

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
	pico8: Pico8VM,
}

impl EmulatorApp {
	pub fn new(cc: &CreationContext) -> EmulatorApp {
		let mut fb_pool = FramebufferPool::new(128, 128);
		let fb_tex = cc.egui_ctx.load_texture("framebuffer", fb_pool.from_color(Color32::MAGENTA), FRAMEBUFFER_OPTS);
		
		let mut pico8 = Pico8VM::new().unwrap();
		pico8.load(b"
			printh(\"Filling\")
			for off = 0,64*128 do
			  poke(0x6000 + off, off % 256)
			  if off % 256 == 255 then
			    flip()
			  end
			end
			printh(\"Done!\")
		");
		
		Self {
			fb_pool,
			fb_tex,
			frame: 0,
			last_frames: [Instant::now(); 10],
			pico8,
		}
	}
}

impl eframe::App for EmulatorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.pico8.run();
		
		self.fb_tex.set(self.fb_pool.from_iter(
			self.pico8.env()
			          .memory
			          .screen()
			          .iter()
			          .map(|byte| [PALETTE[*byte as usize >> 4], PALETTE[*byte as usize & 0x0F]])
			          .flatten()
			          .map(|color| {
				          let (r, g, b) = color.rgb();
				          Color32::from_rgb(r, g, b)
			          })
		), FRAMEBUFFER_OPTS);
		
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
				ui.label(format!("Frame {}", self.frame));
				ui.label(format!("FPS: {:>4.0}", self.last_frames.len() as f32 / self.last_frames.last().unwrap().elapsed().as_secs_f32()));
			});
		
		
		self.last_frames.rotate_right(1);
		self.last_frames[0] = Instant::now();
		
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
