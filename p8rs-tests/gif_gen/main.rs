use std::fmt::Write;
use std::fs;
use std::path::Path;
use anyhow::Result;
use gif::{Encoder, Frame};
use p8rs::vm::font::Font;
use p8rs::vm::palette::PALETTE;
use p8rs_tests::log::Log;
use p8rs_tests::summary::Summary;
use p8rs_tests::TMP_DIR;
use p8rs_types::p8scii;
use p8rs_types::p8scii::LossyIteratorEx;

const PADDING_X: usize = 8;
const PADDING_Y: usize = 16;
const CELL_WIDTH: usize = 128 + PADDING_X * 2;
const CELL_HEIGHT: usize = 128 + PADDING_Y * 2;
const DELAY: u16 = 5; // * 10ms
const FONT: Font = Font::SYSTEM;

fn main() {
	let args = std::env::args().skip(1).collect::<Vec<_>>();
	let do_summary = !args.iter().any(|arg| arg == "--no-summary");
	
	let palette: Vec<_> = PALETTE.iter()
	                             .flat_map(|col| { let (r, g, b) = col.rgb(); [r, g, b] })
	                             .collect();
	
	let results = load_results(TMP_DIR).expect("Failed to load results from tmp dir");
	let gif_path = Path::new(TMP_DIR).join("tests.gif");
	let mut gif = fs::File::create(&gif_path).expect("Could not create gif file");
	let extra_cells = if do_summary { 1 } else { 0 };
	let grid_height = (results.len() + extra_cells).isqrt();
	let grid_width = (results.len() + extra_cells).div_ceil(grid_height);
	let image_width = u16::try_from(grid_width * CELL_WIDTH).expect("Output too large to fit in a gif file");
	let image_height = u16::try_from(grid_height * CELL_HEIGHT).expect("Output too large to fit in a gif file");
	let mut encoder = Encoder::new(&mut gif, image_width, image_height, &palette).unwrap();
	encoder.set_repeat(gif::Repeat::Infinite).unwrap();
	
	for result in results.iter() {
		println!("{} {} {}", result.summary.orig_cart_path, result.pico8_logs.len(), result.p8rs_logs.len());
	}
	
	let mut last_step = vec![usize::MAX; results.len()];
	let mut screen_cache = vec![ResultCache::new(); results.len()];
	let frames = results.iter().map(|res| res.steps).max().unwrap_or(0);
	
	let mut frame = Frame::default();
	frame.dispose = gif::DisposalMethod::Any;
	frame.delay = DELAY;
	frame.width = image_width;
	frame.height = image_height;
	frame.buffer = vec![0_u8; image_width as usize * image_height as usize].into();
	
	if do_summary {
		draw_summary(
			frame.buffer.to_mut(),
			(results.len() % grid_width) * CELL_WIDTH,
			(results.len() / grid_width) * CELL_HEIGHT,
			grid_width * CELL_WIDTH,
			&results,
		);
	}
	
	for frame_id in 0..frames {
		println!("Frame {}/{frames}", frame_id + 1);
		
		for (cell_id, result) in results.iter().enumerate() {
			let step = frame_id * result.steps() / frames;
			if last_step[cell_id] != step {
				last_step[cell_id] = step;
				draw_step(
					frame.buffer.to_mut(),
					(cell_id % grid_width) * CELL_WIDTH,
					(cell_id / grid_width) * CELL_HEIGHT,
					grid_width * CELL_WIDTH,
					&mut screen_cache[cell_id],
					&result,
					step,
				);
			}
		}
		
		encoder.write_frame(&frame).expect("Failed to write frame to gif file");
	}
	
	drop(encoder);
	
	opener::open(gif_path).expect("Failed to open gif file");
}

fn load_results(path: impl AsRef<Path>) -> Result<Vec<TestResults>> {
	Ok(
		fs::read_dir(path)?
			.inspect(|res| if let Err(err) = res { eprintln!("Couldn't list file: {}", err) })
			.flatten()
			.filter(|entry|
				entry.path().extension().is_some_and(|ext| ext.to_str() == Some("json"))
					&& entry.metadata().is_ok_and(|md| md.is_file()))
			.flat_map(|entry|
				TestResults::from_json(entry.path())
					.inspect_err(|err| eprintln!("Failed to load {}: {err}", entry.path().display())))
			.collect()
	)
}

#[derive(Debug, Clone)]
struct TestResults {
	summary: Summary<'static>,
	pico8_logs: Vec<Log>,
	p8rs_logs: Vec<Log>,
	steps: usize,
	valid: usize,
}

impl TestResults {
	fn from_json(path: impl AsRef<Path>) -> Result<Self> {
		let summary: Summary = serde_json::from_reader(fs::File::open(path)?)?;
		let pico8_logs: Vec<Log> = Log::parse(&fs::read_to_string(summary.pico8.log_path.as_ref())?);
		let p8rs_logs: Vec<Log> = Log::parse(&fs::read_to_string(summary.p8rs.log_path.as_ref())?);
		let steps = pico8_logs.len().max(p8rs_logs.len());
		let valid = (0..steps).position(|step| pico8_logs.get(step) != p8rs_logs.get(step)).unwrap_or(steps);
		
		Ok(TestResults {
			summary,
			pico8_logs,
			p8rs_logs,
			steps,
			valid,
		})
	}
	
	fn steps(&self) -> usize {
		self.pico8_logs.len().max(self.p8rs_logs.len())
	}
}

#[derive(Debug, Clone)]
struct ResultCache {
	screen: [u8; 128 * 128],
	next_print: usize,
}

impl ResultCache {
	const TEXT_PAD_X: usize = 2;
	const TEXT_PAD_Y: usize = 1;
	
	const fn new() -> Self {
		Self {
			screen: [0; 128 * 128],
			next_print: Self::TEXT_PAD_Y,
		}
	}
	
	fn print_str(&mut self, fg: u8, bg: u8, text: &str) {
		let p8scii: Vec<u8> = p8scii::from_str(text).lossy().collect();
		let mut x_pos = 0;
		let chunker = move |_: &u8, &ch: &u8| {
			let c_width = FONT.width_chr(ch) as usize;
			
			if ch == b'\n' {
				x_pos = 0;
				false
			} else {
				x_pos += c_width;
				if x_pos > 128 - Self::TEXT_PAD_X * 2 - c_width {
					x_pos = 0;
					false
				} else {
					true
				}
			}
		};
		
		for line in p8scii.chunk_by(chunker) {
			let line_height = FONT.height() as usize;
			
			if let Some(overflow) = self.next_print.checked_sub(128 - Self::TEXT_PAD_Y - line_height) {
				self.screen.rotate_left(128 * overflow);
				self.next_print -= overflow;
			}
			
			if self.next_print <= Self::TEXT_PAD_Y {
				self.screen[0 .. 128].fill(bg);
			}
			
			self.screen[self.next_print * 128 .. (self.next_print + line_height + 1) * 128].fill(bg);
			draw_p8scii(&mut self.screen, Self::TEXT_PAD_X, self.next_print, 128, fg, bg, line.iter().copied());
			
			self.next_print += FONT.height() as usize;
		}
	}
	
	fn print_scr(&mut self, scr_pal: [u8; 16], pixels: &[u8; 128 * 128]) {
		self.next_print = Self::TEXT_PAD_Y;
		
		for (i, p) in pixels.iter().enumerate() {
			self.screen[i] = to_sys_pal(scr_pal[(p & 0x0F) as usize]);
		}
	}
}

fn to_sys_pal(index: u8) -> u8 {
	let nib = index & 0x0F;
	if index < 128 {
		nib
	} else {
		nib + 16
	}
}

fn draw_step(framebuffer: &mut [u8], cell_x: usize, cell_y: usize, stride: usize, cache: &mut ResultCache, results: &TestResults, step: usize) {
	let pos = |x: usize, y: usize| (cell_x + x) + (cell_y + y) * stride;
	let name = &results.summary.orig_cart_path;
	let error = step >= results.valid;
	let step_name = results.pico8_logs.get(step)
	                                  .or(results.p8rs_logs.get(step))
	                                  .and_then(Log::name)
	                                  .unwrap_or("<unknown>");
		
	for row in 0..CELL_HEIGHT {
		framebuffer[pos(0, row) .. pos(CELL_WIDTH, row)].fill(0);
	}
	
	match results.pico8_logs.get(step) {
		Some(Log::SCR(_, scr_pal, pixels)) => {
			cache.print_scr(*scr_pal, pixels.as_flattened().try_into().unwrap());
		},
		Some(Log::TEST(_, text)) => {
			cache.print_str(7, 0, text);
		},
		Some(Log::MEM(_, offset, memory)) => {
			for (c_id, chunk) in memory.chunks(8).enumerate() {
				let mut text = format!("0X{:04x}:  ", offset.wrapping_add(c_id as u16));
				for (pos, byte) in chunk.iter().enumerate() {
					write!(text, "{:02x}", byte).unwrap();
					if (pos + 1) % 4 == 0 {
						text.push_str("  ");
					} else if (pos + 1) % 2 == 0 {
						text.push_str(" ");
					}
				}
				
				cache.print_str(7, 0, text.trim());
			}
		},
		_ => {}
	}
	
	let border_col = if error { 8 } else { 6 };
	let subtext_col = if error { 8 } else { 7 };
	let step_counter = format!("{}/{}", step + 1, results.steps());
	let max_chars = 128 / FONT.width() as usize;
	
	if step_name.len() + step_counter.len() > max_chars {
		let mut p8scii: Vec<_> = p8scii::from_str(step_name).lossy().collect();
		p8scii.truncate(max_chars - step_counter.len() - 3);
		p8scii.push(p8scii::from_char('…').unwrap().unwrap());
		draw_p8scii(framebuffer, cell_x + PADDING_X, cell_y + PADDING_Y + 132, stride, subtext_col, 0, p8scii);
	} else {
		draw_str(framebuffer, cell_x + PADDING_X, cell_y + PADDING_Y + 132, stride, subtext_col, 0, step_name);
	}
	
	draw_str(framebuffer, cell_x + PADDING_X, cell_y + PADDING_Y - 8, stride, 7, 0, name);
	draw_str(framebuffer, cell_x + PADDING_X + 129 - step_counter.len() * 4, cell_y + PADDING_Y + 132, stride, 7, 0, &step_counter);
	draw_rect(framebuffer, cell_x + PADDING_X - 1, cell_y + PADDING_Y - 1, stride, 130, 130, border_col);
	for row in 0..128 {
		framebuffer[pos(PADDING_X, PADDING_Y + row) .. pos(PADDING_X + 128, PADDING_Y + row)].copy_from_slice(&cache.screen[row * 128 .. (row + 1) * 128]);
	}
}

fn draw_summary(framebuffer: &mut [u8], orig_x: usize, orig_y: usize, stride: usize, results: &[TestResults]) {
	let tests = results.len();
	let steps: usize = results.iter().map(|res| res.steps).sum();
	let valid: usize = results.iter().map(|res| res.valid).sum();
	let fails = steps - valid;
	let lh = FONT.height() as usize * 2;
	
	draw_str_large(framebuffer, orig_x + PADDING_X, orig_y + PADDING_Y + lh * 0, stride, 7, 0, "「 test summary 」");
	draw_str_large(framebuffer, orig_x + PADDING_X, orig_y + PADDING_Y + lh * 2, stride, 6, 0, &format!("   tests: {tests}"));
	draw_str_large(framebuffer, orig_x + PADDING_X, orig_y + PADDING_Y + lh * 3, stride, 6, 0, &format!("   steps: {steps}"));
	draw_str_large(framebuffer, orig_x + PADDING_X, orig_y + PADDING_Y + lh * 4, stride, 26, 0, &format!("  passed: {valid}"));
	draw_str_large(framebuffer, orig_x + PADDING_X, orig_y + PADDING_Y + lh * 5, stride, 14, 0, &format!("  failed: {fails}"));
	draw_str_large(framebuffer, orig_x + PADDING_X, orig_y + PADDING_Y + lh * 7, stride, 7, 0, &format!("  result: {:.0}%", valid as f32 / steps as f32 * 100.0));
}

fn draw_rect(framebuffer: &mut [u8], orig_x: usize, orig_y: usize, stride: usize, width: usize, height: usize, col: u8) {
	let pos = |x: usize, y: usize| (orig_x + x) + (orig_y + y) * stride;
	
	framebuffer[pos(0, 0) .. pos(width, 0)].fill(col);
	for row in 1..height {
		framebuffer[pos(0, row)] = col;
		framebuffer[pos(width - 1, row)] = col;
	}
	framebuffer[pos(0, height - 1) .. pos(width, height - 1)].fill(col);
}

fn draw_str(framebuffer: &mut [u8], x: usize, y: usize, stride: usize, fg: u8, bg: u8, text: impl AsRef<str>) {
	draw_p8scii(framebuffer, x, y, stride, fg, bg, p8scii::from_str(text.as_ref()).lossy())
}

fn draw_p8scii(framebuffer: &mut [u8], mut x: usize, y: usize, stride: usize, fg: u8, bg: u8, text: impl IntoIterator<Item = u8>) {
	for char in text {
		let bitmap = FONT.char(char);
		let width = FONT.width_chr(char) as usize;
		let height = FONT.height() as usize;
		
		for row in 0..height {
			for col in 0..width {
				framebuffer[(x + col) + (y + row) * stride] = if bitmap[row] & 1 << col == 0 { bg } else { fg };
			}
		}
		
		x += width;
	}
}

fn draw_str_large(framebuffer: &mut [u8], x: usize, y: usize, stride: usize, fg: u8, bg: u8, text: impl AsRef<str>) {
	draw_p8scii_large(framebuffer, x, y, stride, fg, bg, p8scii::from_str(text.as_ref()).lossy())
}

fn draw_p8scii_large(framebuffer: &mut [u8], mut x: usize, y: usize, stride: usize, fg: u8, bg: u8, text: impl IntoIterator<Item = u8>) {
	for char in text {
		let bitmap = FONT.char(char);
		let width = FONT.width_chr(char) as usize;
		let height = FONT.height() as usize;
		
		for row in 0..height {
			for col in 0..width {
				let color = if bitmap[row] & 1 << col == 0 { bg } else { fg };
				framebuffer[(x + col * 2) + (y + row * 2) * stride] = color;
				framebuffer[(x + col * 2 + 1) + (y + row * 2) * stride] = color;
				framebuffer[(x + col * 2) + (y + row * 2 + 1) * stride] = color;
				framebuffer[(x + col * 2 + 1) + (y + row * 2 + 1) * stride] = color;
			}
		}
		
		x += width * 2;
	}
}
