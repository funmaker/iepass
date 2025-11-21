use core::ops::{Range, RangeInclusive};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Vector<T> {
	pub x: T,
	pub y: T,
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
