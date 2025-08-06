#![feature(arc_is_unique)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod framebuffer_pool;

use std::io::{Cursor};
use std::time::Instant;
use eframe::{egui, CreationContext};
use eframe::epaint::TextureHandle;
use egui::{Color32, ColorImage, Frame, ImageSource};
use egui::load::SizedTexture;
use crate::framebuffer_pool::{FramebufferPool, FRAMEBUFFER_OPTS};

fn main() -> eframe::Result {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 512.0]),
		..Default::default()
	};
	
	let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
	let source = rodio::Decoder::new(Cursor::new(include_bytes!("../../assets/pszczoly.wav"))).unwrap();
	stream_handle.mixer().add(source);
	
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
	frame: f32,
	last_frames: [Instant; 10],
	mask: [u8; 207],
}

impl EmulatorApp {
	pub fn new(cc: &CreationContext) -> Self {
		let mut fb_pool = FramebufferPool::new(128, 128);
		let fb_tex = cc.egui_ctx.load_texture("framebuffer", fb_pool.from_color(Color32::MAGENTA), FRAMEBUFFER_OPTS);
		
		Self {
			fb_pool,
			fb_tex,
			frame: 0.0,
			mask: MASK,
			last_frames: [Instant::now(); 10],
		}
	}
}

impl eframe::App for EmulatorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.fb_tex.set(self.fb_pool.from_map(|x, y| {
			fn b2i (b: bool) -> usize { if b { 1 } else { 0 } }
			
			let delay = (0..5).find_map(|d| {
				let x = (x as isize) / 5 - 1;
				let y = (y as f32 + (x as f32 / 4.0 - (self.frame - d as f32 * 10.0) / 15.0).sin() * 6.0) as isize / 5 - 8;
				if x < 0 || x >= 23 || y < 0 || y >= 9 {
					return None;
				}
				let value = self.mask[(x as usize + y as usize * 23) % self.mask.len()];
				if value == 0 { None } else { Some((d, value)) }
			});
			
			match delay {
				Some((d, 1)) => Color32::GRAY.linear_multiply(1.0 / (d + 1) as f32),
				Some((d, 2)) => Color32::BLUE.linear_multiply(1.0 / (d + 1) as f32),
				_ => {
					let x = x as f32 + 0.1;
					let y = y as f32 + 0.1;
					if (
						b2i((x + self.frame * 0.8 + 100.0) % 16.0 > 8.0)
							+ b2i((
							y + (self.frame / 60.0).sin() * 30.0 / 3.0
								+ (x / 10.0 + self.frame / 30.0).sin() * 5.0 + 100.0
						) % 16.0 > 8.0)
					) % 2 == 0 {
						Color32::MAGENTA.gamma_multiply((x / 10.0 + self.frame / 30.0).sin() * 0.4 + 0.6)
					} else {
						Color32::BLACK
					}
				}
			}
		}), FRAMEBUFFER_OPTS);
		
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
				ui.label(format!("Frame {:.1}, FPS: {:>4.0}", self.frame, self.last_frames.len() as f32 / self.last_frames.last().unwrap().elapsed().as_secs_f32()));
				if ui.button("XD").clicked() {
					self.mask.chunks_exact_mut(23).for_each(|row| row.rotate_left(1));
				}
			});
		
		ctx.request_repaint();
		
		let now = Instant::now();
		let dt = now - self.last_frames[0];
		self.frame += 60.0 * dt.as_secs_f32();
		self.last_frames.rotate_right(1);
		self.last_frames[0] = now;
	}
}

const MASK: [u8; 23 * 9] = [
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 2, 0, 2, 2, 2, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0,
	0, 2, 0, 2, 2, 2, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0,
	0, 2, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0,
	0, 2, 0, 2, 2, 2, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
