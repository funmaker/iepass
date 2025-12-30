use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process;
use std::env;
use getopts::Options;
use anyhow::Result;
use gif::{Encoder, Frame};
use p8rs::macros::p8;
use p8rs::vm::font::Font;
use p8rs::vm::palette::PALETTE;
use p8rs_tests::log::Log;
use p8rs_tests::summary::Summary;
use p8rs_types::p8scii;
use p8rs_types::p8scii::LossyIteratorEx;

const PADDING_X: usize = 8;
const PADDING_Y: usize = 16;
const CELL_WIDTH: usize = 128 + PADDING_X * 2;
const CELL_HEIGHT: usize = 128 + PADDING_Y * 2;
const FONT: Font = Font::SYSTEM;

fn main() {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflag("h", "help", "Print this help menu");
	opts.optflag("s", "summary", "Include test summary");
	opts.optflag("o", "open", "Open generated gif");
	opts.optopt("f", "fps", "Target framerate of the gif", "20");
	opts.optopt("", "scale", "Gif scale", "1");
	
	let matches = opts.parse(&args[1..]).expect("Could not parse command line arguments");
	let help = matches.opt_present("help");
	let summary = matches.opt_present("summary");
	let open = matches.opt_present("open");
	let delay = match matches.opt_get::<f32>("fps").expect("Could not parse fps argument") {
		Some(fps) if fps.is_normal() => (100.0 / fps).round().clamp(1.0, u16::MAX as f32) as u16,
		None => 10,
		Some(fps) => panic!("Invalid fps value: {fps}"),
	};
	let scale = match matches.opt_get::<usize>("scale").expect("Could not parse scale argument") {
		Some(scale) if scale > 0 => scale,
		None => 1,
		Some(scale) => panic!("Invalid scale value: {scale}"),
	};
	let error = matches.free.len() != 2;
	
	if help || error {
		let brief = format!("Usage: gif-gen p8rs-tests/tmp p8rs-tests/tmp/tests.gif [Options...]");
		if error {
			eprint!("{}", opts.usage(&brief));
			process::exit(-1);
		} else {
			print!("{}", opts.usage(&brief));
			return;
		}
	}
	
	let [tmp_path, output_path] = matches.free.try_into().unwrap();
	
	let palette: Vec<_> = PALETTE.iter()
	                             .flat_map(|col| { let (r, g, b) = col.rgb(); [r, g, b] })
	                             .collect();
	
	let mut results = load_results(tmp_path).expect("Failed to load results from tmp dir");
	results.sort_by(|a, b| a.summary.orig_cart_path.cmp(&b.summary.orig_cart_path));
	
	let mut gif = fs::File::create(&output_path).expect("Could not create gif file");
	let extra_cells = if summary { 1 } else { 0 };
	let grid_height = (results.len() + extra_cells).isqrt();
	let grid_width = (results.len() + extra_cells).div_ceil(grid_height);
	let fb_width = grid_width * CELL_WIDTH;
	let fb_height = grid_height * CELL_HEIGHT;
	let im_width = u16::try_from(fb_width * scale).expect("Output too large to fit in a gif file");
	let im_height = u16::try_from(fb_height * scale).expect("Output too large to fit in a gif file");
	let mut encoder = Encoder::new(&mut gif, im_width, im_height, &palette).unwrap();
	encoder.set_repeat(gif::Repeat::Infinite).unwrap();
	
	let mut last_step = vec![usize::MAX; results.len()];
	let mut screen_cache = vec![ResultCache::new(); results.len()];
	let mut framebuffer = vec![0_u8; fb_width * fb_height];
	let frames = results.iter().map(|res| res.steps).max().unwrap_or(0);
	
	let mut frame = Frame::default();
	frame.dispose = gif::DisposalMethod::Any;
	frame.delay = delay;
	frame.width = im_width;
	frame.height = im_height;
	frame.buffer = vec![0_u8; framebuffer.len() * scale.pow(2)].into();
	
	if summary {
		let summary_pos = grid_height * grid_width - 1;
		draw_summary(
			&mut framebuffer,
			(summary_pos % grid_width) * CELL_WIDTH,
			(summary_pos / grid_width) * CELL_HEIGHT,
			fb_width,
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
					&mut framebuffer,
					(cell_id % grid_width) * CELL_WIDTH,
					(cell_id / grid_width) * CELL_HEIGHT,
					fb_width,
					&mut screen_cache[cell_id],
					&result,
					step,
				);
			}
		}
		
		for y in 0..fb_height {
			for x in 0..fb_width {
				for ys in 0..scale {
					for xs in 0..scale {
						frame.buffer.to_mut()[
							y * im_width as usize * scale
							+ ys * im_width as usize
							+ x * scale
							+ xs
						] = framebuffer[y * fb_width + x];
					}
				}
			}
		}
		
		encoder.write_frame(&frame).expect("Failed to write frame to gif file");
	}
	
	drop(encoder);
	
	if open {
		opener::open(output_path).expect("Failed to open gif file");
	}
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
		let dir = path.as_ref().parent().unwrap();
		let summary: Summary = serde_json::from_reader(fs::File::open(path.as_ref())?)?;
		let pico8_logs: Vec<Log> = Log::parse(&fs::read_to_string(dir.join(summary.pico8.log_name.as_ref()))?);
		let p8rs_logs: Vec<Log> = Log::parse(&fs::read_to_string(dir.join(summary.p8rs.log_name.as_ref()))?);
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
			draw_p8scii(&mut self.screen, Self::TEXT_PAD_X, self.next_print, 128, fg, bg, None, line.iter().copied());
			
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
	
	let border_col = if error { 8 } else  { 6 };
	let subtext_col = if error { 8 } else { 7 };
	let step_counter = format!("{}/{}", step + 1, results.steps());
	
	if results.valid == results.steps {
		draw_str(framebuffer, cell_x + PADDING_X + 129 - 4 * 4, cell_y + PADDING_Y - 8, stride, 26, 0, None, "done");
	} else {
		draw_str(framebuffer, cell_x + PADDING_X + 129 - 4 * 4, cell_y + PADDING_Y - 8, stride, 14, 0, None, "fail");
	};
	
	draw_str(framebuffer, cell_x + PADDING_X, cell_y + PADDING_Y - 8, stride, 7, 0, Some(130 - 16), name);
	draw_str(framebuffer, cell_x + PADDING_X, cell_y + PADDING_Y + 132, stride, subtext_col, 0, Some(130 - step_counter.len() as u8 * 4), step_name);
	draw_str(framebuffer, cell_x + PADDING_X + 129 - step_counter.len() * 4, cell_y + PADDING_Y + 132, stride, 7, 0, None, &step_counter);
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

fn draw_str(framebuffer: &mut [u8], x: usize, y: usize, stride: usize, fg: u8, bg: u8, max_len: Option<u8>, text: impl AsRef<str>) {
	draw_p8scii(framebuffer, x, y, stride, fg, bg, max_len, p8scii::from_str(text.as_ref()).lossy())
}

fn draw_p8scii(framebuffer: &mut [u8], x: usize, y: usize, stride: usize, fg: u8, bg: u8, max_len: Option<u8>, text: impl IntoIterator<Item = u8>) {
	const ELLIPSIS_CH: u8 = p8!('…');
	let ellipsis_width = FONT.width_chr(ELLIPSIS_CH);
	
	let mut len = 0;
	for char in text {
		let width = FONT.width_chr(char);
		
		if max_len.is_some_and(|max_len| len as u8 + width > max_len - ellipsis_width) {
			draw_p8scii_char(framebuffer, x + len, y, stride, fg, bg, ELLIPSIS_CH);
			return;
		} else {
			len += draw_p8scii_char(framebuffer, x + len, y, stride, fg, bg, char) as usize;
		}
	}
}

fn draw_p8scii_char(framebuffer: &mut [u8], x: usize, y: usize, stride: usize, fg: u8, bg: u8, char: u8) -> u8 {
	let bitmap = FONT.char(char);
	let width = FONT.width_chr(char);
	let height = FONT.height();
	
	for row in 0..height as usize {
		for col in 0..width as usize {
			framebuffer[(x + col) + (y + row) * stride] = if bitmap[row] & 1 << col == 0 { bg } else { fg };
		}
	}
	
	width
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
