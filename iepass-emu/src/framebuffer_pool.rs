use std::sync::Arc;
use eframe::epaint::{Color32, ColorImage};
use eframe::epaint::textures::{TextureFilter, TextureOptions, TextureWrapMode};

pub const FRAMEBUFFER_OPTS: TextureOptions = TextureOptions {
	magnification: TextureFilter::Nearest,
	minification: TextureFilter::Linear,
	wrap_mode: TextureWrapMode::Repeat,
	mipmap_mode: Some(TextureFilter::Linear),
};

pub struct FramebufferPool {
	width: usize,
	height: usize,
	pool: Vec<Arc<ColorImage>>,
}

impl FramebufferPool {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			width,
			height,
			pool: Vec::new(),
		}
	}
	
	pub fn get(&mut self, fill: Color32) -> &mut Arc<ColorImage> {
		// NLL Case 3#
		let pos = self.pool
			.iter_mut()
			.position(|image| Arc::is_unique(image))
			.unwrap_or_else(|| {
				self.pool.push(Arc::new(ColorImage::new(
					[self.width, self.height],
					vec![fill; self.width * self.height],
				)));
				
				self.pool.len() - 1
			});
		
		&mut self.pool[pos]
	}
	
	pub fn from_color(&mut self, color: Color32) -> Arc<ColorImage> {
		let image = self.get(color);
		
		let inner = Arc::get_mut(image).unwrap();
		inner.pixels.fill(color);
		
		image.clone()
	}
	
	pub fn from_slice(&mut self, pixels: &[Color32]) -> Arc<ColorImage> {
		assert_eq!(pixels.len(), self.width * self.height, "Invalid pixel count. Expected {}, got {}", self.width * self.height, pixels.len());
		
		let image = self.get(Color32::TRANSPARENT);
		
		let inner = Arc::get_mut(image).unwrap();
		inner.pixels.copy_from_slice(pixels);
		
		image.clone()
	}
	
	pub fn from_iter(&mut self, pixels: impl Iterator<Item = Color32>) -> Arc<ColorImage> {
		let size = self.width * self.height;
		let image = self.get(Color32::TRANSPARENT);
		
		let inner = Arc::get_mut(image).unwrap();
		inner.pixels.clear();
		inner.pixels.extend(pixels.take(size));
		
		assert_eq!(inner.pixels.len(), size, "Invalid pixel count. Expected {}, got {}", size, inner.pixels.len());
		
		image.clone()
	}
	
	pub fn from_map<F>(&mut self, pixels: F) -> Arc<ColorImage>
	where F: Fn(usize, usize) -> Color32 {
		let height = self.height;
		let width = self.width;
		let image = self.get(Color32::TRANSPARENT);
		
		let inner = Arc::get_mut(image).unwrap();
		for y in 0..height {
			for x in 0..width {
				inner.pixels[x + y * width] = pixels(x, y);
			}
		}
		
		image.clone()
	}
}
