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

pub trait PaintRange {
	fn cam_range(self, clip: Range<u8>, camera: i16) -> impl IntoIterator<Item=u8> + Clone;
	fn abs_range(self, clip: Range<u8>) -> impl IntoIterator<Item=u8> + Clone;
}

macro_rules! impl_clip {
    () => {};
	($typ:ty $(, $rest:ty )*) => {
		impl PaintRange for Range<$typ> {
			fn cam_range(self, clip: Range<u8>, camera: i16) -> impl IntoIterator<Item=u8> + Clone {
				let start = (self.start as i16).wrapping_sub(camera).clamp(clip.start as i16, 255) as u8;
				let end = (self.end as i16).wrapping_sub(camera).clamp(0, clip.end as i16) as u8;
				
                start..end
			}

            fn abs_range(self, clip: Range<u8>) -> impl IntoIterator<Item=u8> + Clone {
				let start = self.start.clamp(clip.start as $typ, 255) as u8;
				let end = self.end.clamp(0, clip.end as $typ) as u8;
				
                start..end
            }
		}
		
		impl PaintRange for RangeInclusive<$typ> {
			fn cam_range(self, clip: Range<u8>, camera: i16) -> impl IntoIterator<Item=u8> + Clone {
				let start = (*self.start() as i16).wrapping_sub(camera).clamp(clip.start as i16, 255) as u8;
				let end = (*self.end() as i16).wrapping_sub(camera).wrapping_add(1).clamp(0, clip.end as i16) as u8;
				
                start..end
			}

            fn abs_range(self, clip: Range<u8>) -> impl IntoIterator<Item=u8> + Clone {
				let start = (*self.start()).clamp(clip.start as $typ, 255) as u8;
				let end = (*self.end()).wrapping_add(1).clamp(0, clip.end as $typ) as u8;
				
                start..end
            }
		}
		
		impl PaintRange for $typ {
			fn cam_range(self, clip: Range<u8>, camera: i16) -> impl IntoIterator<Item=u8> + Clone {
                let value = (self as i16).wrapping_sub(camera);
                if value >= clip.start as i16 && value < clip.end as i16 {
                    Some(value as u8)
                } else {
                    None
                }
			}

            fn abs_range(self, clip: Range<u8>) -> impl IntoIterator<Item=u8> + Clone {
                if self >= clip.start as $typ && self < clip.end as $typ {
                    Some(self as u8)
                } else {
                    None
                }
            }
		}
		
		impl_clip!($( $rest ),*);
	}
}

impl_clip!(u8, i16);
