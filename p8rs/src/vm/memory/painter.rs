mod mode;
mod callback;
mod utils;

use core::ops::Range;

pub use callback::{PainterCallback, CallbackResult, Noop};
pub use mode::{PainterMode, PenMode, SpriteMode, TextMode};
pub use utils::PaintRange;
use super::{Memory, Screen};
use utils::Vector;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Painter<Mode = PenMode> {
	screen: Screen,
	clip_x: Range<u8>,
	clip_y: Range<u8>,
	camera: Vector<i16>,
	fill: Option<[[bool; 4]; 4]>,
	mode: Mode,
}

impl<'m> Painter<PenMode> {
	pub(super) fn new(screen: Screen, memory: &'m mut Memory) -> Self {
		let mut ms = memory.machine_state();
		let clip = *ms.clip_rect();
		let camera = ms.get_camera_position();
		let fill = ms.fill_pattern().expand();
		
		Painter {
			screen,
			clip_x: clip[0] .. clip[2].min(128),
			clip_y: clip[1] .. clip[3].min(128),
			camera: Vector::from(camera),
			fill,
			mode: PenMode::new(memory),
		}
	}
}

impl<'m, Mode: PainterMode> Painter<Mode> {
	pub fn set_fill(mut self, fill: Option<[[bool; 4]; 4]>) -> Self {
		self.fill = fill;
		self
	}
	
	pub fn sprite_mode(self, memory: &mut Memory) -> Painter<SpriteMode> {
		Painter {
			mode: SpriteMode::new(memory),
			screen: self.screen,
			clip_x: self.clip_x,
			clip_y: self.clip_y,
			camera: self.camera,
			fill: self.fill,
		}
	}
	
	pub fn text_mode(self, memory: &mut Memory, bg_color: Option<u8>) -> Painter<TextMode> {
		Painter {
			mode: TextMode::new(memory, bg_color),
			screen: self.screen,
			clip_x: self.clip_x,
			clip_y: self.clip_y,
			camera: self.camera,
			fill: self.fill,
		}
	}
	
	pub fn screen(&self) -> Screen {
		self.screen
	}
	
	pub fn to_abs(&self, x: i16, y: i16) -> (i16, i16) {
		let x = x.saturating_sub(self.camera.x);
		let y = y.saturating_sub(self.camera.y);
		
		(x, y)
	}
	
	pub fn paint(&self, memory: &mut Memory, x: impl PaintRange, y: impl PaintRange) -> &Self {
		self.paint_tex(memory, x, y, Noop)
	}
	
	pub fn paint_tex(&self, memory: &mut Memory, x: impl PaintRange, y: impl PaintRange, callback: impl PainterCallback) -> &Self {
		let x = x.cam_range(self.clip_x.clone(), self.camera.x);
		let y = y.cam_range(self.clip_y.clone(), self.camera.y);
		
		for y in y {
			for x in x.clone() {
				self.paint_abs_pixel(memory, x, y, callback);
			}
		}
		
		self
	}
	
	pub fn paint_abs(&self, memory: &mut Memory, x: impl PaintRange, y: impl PaintRange) -> &Self {
		self.paint_abs_tex(memory, x, y, Noop)
	}
	
	pub fn paint_abs_tex(&self, memory: &mut Memory, x: impl PaintRange, y: impl PaintRange, callback: impl PainterCallback) -> &Self {
		let x = x.abs_range(self.clip_x.clone());
		let y = y.abs_range(self.clip_y.clone());
		
		for y in y {
			for x in x.clone() {
				self.paint_abs_pixel(memory, x, y, callback);
			}
		}
		
		self
	}
	
	fn paint_abs_pixel(&self, memory: &mut Memory, x: u8, y: u8, callback: impl PainterCallback) {
		let col = match callback.check(memory, x, y) {
			CallbackResult::Discard => return,
			CallbackResult::Keep => None,
			CallbackResult::Color(col) => Some(col),
		};
		
		let col = if let Some(fill) = self.fill && fill[y as usize % 4][x as usize % 4] {
			self.mode.bg(col)
		} else {
			self.mode.fg(col)
		};
		
		if let Some(col) = col {
			self.screen.set_pixel(memory, x, y, col);
		}
	}
}
