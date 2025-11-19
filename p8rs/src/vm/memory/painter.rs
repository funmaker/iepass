use core::ops::{Not, Range, RangeInclusive};

use crate::vm::memory::machine_state::{FillPatternFlags, Palette};
use crate::vm::memory::Memory;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Painter<'a> {
	memory: &'a mut Memory,
	fg: u8,
	bg: Option<u8>,
	clip_x: Range<i16>,
	clip_y: Range<i16>,
	camera: Vector<i16>,
	fill: Option<[[bool; 4]; 4]>,
}

impl Painter<'_> {
	pub fn new(memory: &mut Memory) -> Painter<'_> {
		let pen_color = *memory.machine_state().pen_color();
		let clip = *memory.machine_state().clip_rect();
		let camera = memory.machine_state().get_camera_position();
		let fill = memory.machine_state().fill_pattern().expand();
		let fill_flags = *memory.machine_state().fill_pattern().flags();
		
		let (fg, bg) = if fill_flags.contains(FillPatternFlags::REMAP_ALL) {
			let pal = *memory.machine_state().palette(Palette::Secondary);
			let color = pal[(pen_color & 0xF) as usize];
			
			(color & 0x0F, color >> 4)
		} else {
			let pal = *memory.machine_state().palette(Palette::Draw);
			let fg = pal[(pen_color & 0xF) as usize];
			let bg = pal[(pen_color >> 4) as usize];
			
			(fg, bg)
		};
		
		let bg = fill_flags.contains(FillPatternFlags::TRANSPARENT)
		                   .not()
		                   .then_some(bg);
		
		Painter {
			memory,
			fg,
			bg,
			clip_x: clip[0] as i16 .. clip[2].min(128) as i16,
			clip_y: clip[1] as i16 .. clip[3].min(128) as i16,
			camera: Vector::from(camera),
			fill,
		}
	}
	
	pub fn paint(&mut self, x: i16, y: i16) {
		let x = x.saturating_sub(self.camera.x);
		let y = y.saturating_sub(self.camera.y);
		
		self.paint_abs_impl(x, y);
	}
	
	pub fn paint_abs(mut self, x: i16, y: i16) {
		if !self.clip_x.contains(&x) || !self.clip_y.contains(&y) { return }
		
		self.paint_abs_impl(x, y);
	}
	
	pub fn paint_range(&mut self, x: impl IntoClip, y: impl IntoClip) {
		let x = range_intersect(x.into_clip(self.camera.x), self.clip_x.clone());
		let y = range_intersect(y.into_clip(self.camera.y), self.clip_y.clone());
		if x.is_empty() || y.is_empty() { return }
		
		for y in y {
			for x in x.clone() {
				self.paint_abs_impl(x, y);
			}
		}
	}
	
	fn paint_abs_impl(&mut self, x: i16, y: i16) {
		let col = if let Some(fill) = self.fill && fill[y as usize % 4][x as usize % 4] {
			if let Some(bg) = self.bg {
				bg
			} else {
				return;
			}
		} else {
			self.fg
		};
		
		self.memory.screen().set_pixel(x, y, col);
	}
}

fn range_intersect<T: Ord>(a: Range<T>, b: Range<T>) -> Range<T> {
	a.start.max(b.start) .. a.end.min(b.end)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Vector<T> {
	x: T,
	y: T,
}

impl<T> From<[T; 2]> for Vector<T> {
	fn from([x, y]: [T; 2]) -> Vector<T> {
		Vector { x, y }
	}
}

pub trait IntoClip {
	fn into_clip(self, camera: i16) -> Range<i16>;
}

macro_rules! impl_clip {
    () => {};
	($typ:ty $(, $rest:ty )*) => {
		impl IntoClip for Range<$typ> {
			fn into_clip(self, camera: i16) -> Range<i16> {
				let start = (self.start as i16).saturating_sub(camera);
				let end = (self.end as i16).saturating_sub(camera);
				
				start..end
			}
		}
		
		impl IntoClip for RangeInclusive<$typ> {
			fn into_clip(self, camera: i16) -> Range<i16> {
				let start = (*self.start() as i16).saturating_sub(camera);
				let end = (*self.end() as i16).saturating_sub(camera).saturating_add(1);
				
				start..end
			}
		}
		
		impl_clip!($( $rest ),*);
	}
}

impl_clip!(u8, i8, i16);
