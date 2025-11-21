mod mode;
mod callback;
mod utils;

use core::ops::Range;

use crate::vm::memory::Memory;
pub use callback::{PainterCallback, Noop};
pub use mode::{PainterMode, PenMode, SpriteMode};
pub use utils::IntoClip;
use utils::Vector;
use crate::vm::memory::painter::callback::CallbackResult;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Painter<'a, Mode = PenMode, CB = Noop> {
	memory: &'a mut Memory,
	clip_x: Range<i16>,
	clip_y: Range<i16>,
	camera: Vector<i16>,
	fill: Option<[[bool; 4]; 4]>,
	mode: Mode,
	callback: CB,
}

impl<'m> Painter<'m, PenMode, Noop> {
	pub fn new(memory: &'m mut Memory) -> Self {
		let clip = *memory.machine_state().clip_rect();
		let camera = memory.machine_state().get_camera_position();
		let fill = memory.machine_state().fill_pattern().expand();
		
		Painter {
			clip_x: clip[0] as i16 .. clip[2].min(128) as i16,
			clip_y: clip[1] as i16 .. clip[3].min(128) as i16,
			camera: Vector::from(camera),
			fill,
			mode: PenMode::new(memory),
			callback: Noop,
			memory,
		}
	}
}

impl<'m, Mode: PainterMode, CB: PainterCallback> Painter<'m, Mode, CB> {
	pub fn with_callback<CB2>(self, callback: CB2) -> Painter<'m, Mode, CB2>
	where CB2: PainterCallback {
		Painter {
			callback,
			memory: self.memory,
			clip_x: self.clip_x,
			clip_y: self.clip_y,
			camera: self.camera,
			fill: self.fill,
			mode: self.mode,
		}
	}
	
	pub fn sprite_mode(self) -> Painter<'m, SpriteMode, CB> {
		Painter {
			mode: SpriteMode::new(self.memory),
			memory: self.memory,
			clip_x: self.clip_x,
			clip_y: self.clip_y,
			camera: self.camera,
			fill: self.fill,
			callback: self.callback,
		}
	}
	
	pub fn to_abs(&self, x: i16, y: i16) -> (i16, i16) {
		let x = x.saturating_sub(self.camera.x);
		let y = y.saturating_sub(self.camera.y);
		
		(x, y)
	}
	
	pub fn paint(&mut self, x: i16, y: i16) -> &mut Self {
		let (x, y) = self.to_abs(x, y);
		
		self.paint_abs_impl(x, y);
		
		self
	}
	
	pub fn paint_abs(&mut self, x: i16, y: i16) -> &mut Self {
		if self.clip_x.contains(&x) && self.clip_y.contains(&y) {
			self.paint_abs_impl(x, y);
		}
		
		self
	}
	
	pub fn paint_range(&mut self, x: impl IntoClip, y: impl IntoClip) -> &mut Self {
		let x = range_intersect(x.into_clip(self.camera.x), self.clip_x.clone());
		let y = range_intersect(y.into_clip(self.camera.y), self.clip_y.clone());
		
		if !x.is_empty() && !y.is_empty() {
			for y in y {
				for x in x.clone() {
					self.paint_abs_impl(x, y);
				}
			}
		}
		
		self
	}
	
	fn paint_abs_impl(&mut self, x: i16, y: i16) {
		let col = match self.callback.check(x as u8, y as u8) {
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
			self.memory.screen().set_pixel(x, y, col);
		}
	}
}

fn range_intersect<T: Ord>(a: Range<T>, b: Range<T>) -> Range<T> {
	a.start.max(b.start) .. a.end.min(b.end)
}
