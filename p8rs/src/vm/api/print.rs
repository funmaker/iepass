use core::alloc::Allocator;
use core::pin::Pin;
use gc_arena::Collect;
use p8rs_piccolo::{BoxSequence, Callback, CallbackReturn, Context, Error, Execution, RuntimeError, RuntimeRef, Sequence, SequencePoll, Stack, String, Value};
use p8rs_types::p8num::P8Num;

use crate::vm::font::Font;
use crate::vm::memory::machine_state::{MiscChipsetFeatureFlags, PrintDefaultsFlags};
use crate::vm::Runtime;

pub fn install_pico8_print<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("print", Callback::from_fn(&ctx, |ctx, _exec, mut stack, rt| {
		let rt = rt.downcast::<Runtime>();
		let (text, mut x, y, mut col): (Value, Option<P8Num>, Option<P8Num>, Option<P8Num>) = stack.consume(ctx).unwrap();
		if y.is_none() {
			col = x;
			x = None;
		}
		
		if let Some((x, y)) = x.zip(y) { *rt.memory.machine_state().cursor_position() = [x.to_integer() as u8, y.to_integer() as u8]; }
		if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
		
		let text = if let Value::String(text) = text {
			text
		} else {
			if let Some(Value::String(str)) = super::base::tostr(ctx, text, None)? {
				str
			} else {
				debug!("[print] Could not convert value to string when printing!");
				String::from_static(ctx.mutation(), "")
			}
		};
		
		trace!("[print] {}", text);
		
		match TextEscapeIterator::new(text).last() {
			None => { // empty string
				// todo: return val
				return Ok(CallbackReturn::Return);
			}
			Some(last_part) => {
				if matches!(last_part, TextPart::UnterminatedEscapeSequence(_)) {
					debug!("[print()] String contained unterminated escape sequence, discarding.");
					// todo: return val
					return Ok(CallbackReturn::Return);
				}
			}
		};
		
		let flags = *rt.memory.machine_state().print_defaults().flags();
		
		if y.is_none() {
			let font_height = get_font_height(rt, flags);
			screen_scroll(rt, font_height, false);
		}
		
		Ok(CallbackReturn::Sequence(BoxSequence::new(ctx.mutation(), PrintSeq {
			skip_frames: 0,
			letter_frame_skip: 0,
			text: TextEscapeIterator::new(text),
			next_char: None,
			stopped: false,
			flags: flags.bits(),
			x_wrapped: false,
			clipping: false,
		})))
	}));
}

fn control_arg_to_number(arg: u8) -> Option<u8> {
	Some(match arg {
		b'0'..=b'9' => arg - b'0',
		b'a'.. => arg - b'a',
		_ => {
			debug!("Unexpected control code argument: {}", arg);
			return None;
		}
	})
}

enum EscapeSequenceAction {
	Nop,
	Stop,
	SkipFrames(usize),
	SetLetterFrameSkip(usize),
	#[allow(unused)]
	ModifyFlags(PrintDefaultsFlags), // TODO:
}


/// Scroll the screen up if needed, so that at least `font_height` new pixels will fit, unless `MiscChipsetFeatureFlags::NO_PRINT_SCROLL` is set.
/// The screen is scrolled up by multiples of `font_height` unless `exact` is true.
fn screen_scroll(rt: &mut Runtime, font_height: u8, exact: bool) {
	if rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL) { return }
	
	let current_y = rt.memory.machine_state().cursor_position()[1];
	let max_y = 128 - font_height;
	if current_y > max_y { return }
	
	let new_y = current_y.saturating_add(font_height);
	if new_y <= max_y { return }
	
	let mut shift = new_y - max_y;
	if !exact {
		shift = ((shift + font_height - 1) / font_height) * font_height;
	}
	
	rt.memory.machine_state().cursor_position()[1] -= shift;
	rt.memory.screen().shift_up(shift, 0);
}

fn cursor_new_line(rt: &mut Runtime, flags: PrintDefaultsFlags, scroll_exact: bool) {
	rt.memory.machine_state().cursor_position()[0] = *rt.memory.machine_state().cursor_home_x();
	
	let font_height = get_font_height(rt, flags);
	screen_scroll(rt, font_height, scroll_exact);
	let current_y = rt.memory.machine_state().cursor_position()[1];
	let new_y = current_y.overflowing_add(font_height).0;
	rt.memory.machine_state().cursor_position()[1] = new_y;
}

fn execute_escape_sequence<'gc>(_ctx: Context<'gc>, rt: &mut Runtime, flags: PrintDefaultsFlags, bytes: &[u8]) -> Result<EscapeSequenceAction, RuntimeError<'gc>> {
	assert!(bytes.len() > 0, "Escape sequence must not be empty");
	
	let escape_code = bytes[0];
	
	Ok(match escape_code {
		0 => EscapeSequenceAction::Stop,
		6 => {
			let arg = bytes[1];
			match arg {
				b'1'..b'9' => {
					let arg = arg - b'1';
					let frames = 1 << arg;
					EscapeSequenceAction::SkipFrames(frames)
				}
				b'd' => {
					let arg = control_arg_to_number(bytes[2]).unwrap(); // TODO: safety
					EscapeSequenceAction::SetLetterFrameSkip(arg as usize)
				}
				_ => {
					debug!("Unimplemented escape sequence! {:?}", bytes);
					EscapeSequenceAction::Nop
				}
			}
		},
		10 => {
			cursor_new_line(rt, flags, false);
			EscapeSequenceAction::Nop
		}
		_ => {
			debug!("Unimplemented escape sequence! {:?}", bytes);
			EscapeSequenceAction::Nop
		}
	})
}

fn get_font_height(rt: &mut Runtime, flags: PrintDefaultsFlags) -> u8 {
	get_font(rt, flags).height()
}

fn get_font(rt: &mut Runtime, flags: PrintDefaultsFlags) -> Font<'_> {
	let use_custom_font = flags.contains(PrintDefaultsFlags::CUSTOM_FONT);
	if use_custom_font {
		Font::new(rt.memory.const_slice(0x5600))
	} else {
		Font::SYSTEM
	}
}

fn draw_letter(_ctx: Context, rt: &mut Runtime, flags: PrintDefaultsFlags, letter: u8, dry_run: bool) -> (i16, i16, bool) {
	// let is_wide = flags.contains(PrintDefaultsFlags::WIDE);
	// let is_tall = flags.contains(PrintDefaultsFlags::TALL);
	// let is_inverted = flags.contains(PrintDefaultsFlags::INVERT);
	// let is_dotty = flags.contains(PrintDefaultsFlags::DOTTY);
	// TODO: Use flags
	
	let pen_color = *rt.memory.machine_state().pen_color();
	
	let font = get_font(rt, flags);
	let char_width = font.width_chr(letter);
	let char_height = font.height();
	let char_font = &font.char(letter);
	
	let overflowed_x = rt.memory.machine_state().cursor_position()[0].overflowing_add(char_width).0 > 128
		&& rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::PRINT_WRAP);
	
	if overflowed_x {
		cursor_new_line(rt, flags, true);
	}
	
	let [cursor_x, cursor_y] = *rt.memory.machine_state().cursor_position();
	
	assert!(char_width <= 8, "Char width cannot be >8");
	assert!(char_height <= 8, "Char height cannot be >8");
	
	if !dry_run {
		for y in 0..char_height {
			let mut font_line = char_font[y as usize];
			for x in 0..char_width {
				let bit = font_line & 1 != 0;
				font_line >>= 1;
				
				if bit {
					let pixel_x = cursor_x.overflowing_add(x).0;
					let pixel_y = cursor_y.overflowing_add(y).0;
					rt.memory.screen().set_pixel(pixel_x as i16, pixel_y as i16, pen_color);
				}
			}
		}
	}
	
	(char_width as i16, char_height as i16, overflowed_x)
}


#[derive(Collect)]
#[collect(no_drop)]
struct PrintSeq<'gc> {
	text: TextEscapeIterator<'gc>,
	/// number of frames that should be skipped right now
	skip_frames: usize,
	/// number of frames that should be skipped before every letter
	letter_frame_skip: usize,
	/// next character to be printer (after possible delay)
	next_char: Option<u8>,
	/// whether a null byte has been encountered (and printing should stop)
	stopped: bool,
	/// current print flags
	flags: u8,
	/// has x wrapped (only if character wrapping is enabled)
	x_wrapped: bool,
	/// should print stop drawing because we ran out either x-space (without character wrapping) or y-space (without line scrolling)
	clipping: bool,
}

impl<'gc> Sequence<'gc> for PrintSeq<'gc> {
	fn poll(
		mut self: Pin<&mut Self>,
		ctx: Context<'gc>,
		_exec: Execution<'gc, '_>,
		_stack: Stack<'gc, '_>,
		rt: RuntimeRef<'_>,
	) -> Result<SequencePoll<'gc>, Error<'gc>> {
		let rt = rt.downcast::<Runtime>();
		
		if !self.stopped {
			if self.skip_frames > 0 {
				self.skip_frames -= 1;
				return Ok(SequencePoll::Yield { to_thread: None, bottom: 0 })
			}
			
			if let Some(char) = self.next_char {
				self.next_char = None;
				
				let chipset_flags = *rt.memory.machine_state().misc_chipset_flags();
				let [cursor_x, cursor_y] = *rt.memory.machine_state().cursor_position();

				if (cursor_y >= 128 && chipset_flags.contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL)) 
				|| (cursor_x >= 128 && !chipset_flags.contains(MiscChipsetFeatureFlags::PRINT_WRAP)) {
					self.clipping = true;
				}
				
				let (w, _, x_overflowed) = draw_letter(ctx, rt, PrintDefaultsFlags::from_bits_truncate(self.flags), char, self.clipping);
				
				if x_overflowed {
					self.x_wrapped = true;
				}
				
				rt.memory.machine_state().cursor_position()[0] = rt.memory.machine_state().cursor_position()[0].overflowing_add(w as u8).0;
			}
			
			let part = self.text.next();
			
			if let Some(part) = part {
				match part {
					TextPart::Character(char) => {
						self.next_char = Some(char);
						self.skip_frames = self.letter_frame_skip;
					}
					TextPart::EscapeSequence(bytes) => {
						match execute_escape_sequence(ctx, rt, PrintDefaultsFlags::from_bits_truncate(self.flags), bytes)? {
							EscapeSequenceAction::Nop => {}
							EscapeSequenceAction::SkipFrames(n) => {
								self.skip_frames = n;
							}
							EscapeSequenceAction::SetLetterFrameSkip(n) => {
								self.letter_frame_skip = n;
							}
							EscapeSequenceAction::Stop => {
								self.stopped = true;
							}
							EscapeSequenceAction::ModifyFlags(new_flags) => {
								self.flags = new_flags.bits();
							}
						}
					}
					TextPart::UnterminatedEscapeSequence(_) => debug!("[PrintSeq] called on a sequence containing UnterminatedEscapeSequence!"),
				}
				
				return Ok(SequencePoll::Pending)
			}
		}
		
		if !rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::NO_PRINT_NEWLINE) {
			cursor_new_line(rt, PrintDefaultsFlags::from_bits_truncate(self.flags), self.x_wrapped);
		}
		
		// todo - return values
		Ok(SequencePoll::Return)
	}
}


// --- TextEscapeIterator ---

#[derive(Collect)]
#[collect(no_drop)]
struct TextEscapeIterator<'gc> {
	text: String<'gc>,
	next_index: usize,
}

impl<'gc> TextEscapeIterator<'gc> {
	pub fn new(text: String<'gc>) -> Self {
		Self {
			next_index: 0,
			text
		}
	}
}

#[derive(Debug, PartialEq)]
enum TextPart<'a> {
	Character(u8),
	EscapeSequence(&'a [u8]),
	UnterminatedEscapeSequence(&'a [u8]),
}

impl<'gc> Iterator for TextEscapeIterator<'gc> {
	type Item = TextPart<'gc>;
	
	fn next(&mut self) -> Option<Self::Item> {
		let text_len = self.text.len();
		
		if self.next_index >= text_len { return None; }
		
		let start_idx = self.next_index;
		let first_char = self.text[start_idx];
		let is_escape = first_char < 16;
		
		self.next_index += 1;
		
		if is_escape {
			match first_char {
				0 | 8 | 9 | 10 | 13 | 14 | 15 => {}, // 1-byte code
				2 | 3 | 4 | 12 => { self.next_index += 1; } // 2-byte
				1 | 5 | 11 => { self.next_index += 2; } // 3-byte
				6 => {
					if self.next_index >= text_len {
						trace!("[TextEscapeIterator] string terminated mid escape sequence (6)");
						return Some(TextPart::UnterminatedEscapeSequence(&self.text.as_bytes()[start_idx..text_len]));
					}
					let command_char = self.text[self.next_index];
					self.next_index += 1;
					
					match command_char {
						b'1'..=b'9' | b'g' | b'h' | b'w' | b't' | b'=' | b'p' | b'i' | b'b' | b'#' | b'$' => {}
						b'-' | b'd' | b'c' | b's' | b'r' | b'x' | b'y' => { self.next_index += 1; }
						b'j' => { self.next_index += 2; }
						b'.' => { self.next_index += 8; }
						b':' => { self.next_index += 16; }
						b'@' => { unimplemented!("TextEscapeIterator \\^@ (6-@) not implemented"); } // poke N bytes
						b'!' => { self.next_index = text_len; } // poke to end
						_ => unimplemented!("TextEscapeIterator: Unimplemented escape code! 6-{}", command_char)
					}
				}
				7 => {
					while self.next_index < text_len && self.text[self.next_index] != b' ' {
						self.next_index += 1;
					}
				}
				_ => unimplemented!("TextEscapeIterator: Unimplemented escape code! {}", first_char)
			}
		}
		
		// next_index will be equal to text_len when the string is fully consumed
		if self.next_index > text_len {
			assert!(is_escape, "Should never run index over if not in escape sequence");
			trace!("[TextEscapeIterator] string terminated mid escape sequence");
			return Some(TextPart::UnterminatedEscapeSequence(&self.text.as_bytes()[start_idx..text_len]));
		}
		
		Some(if is_escape {
			TextPart::EscapeSequence(&self.text.as_bytes()[start_idx..self.next_index])
		} else {
			assert_eq!(self.next_index, start_idx + 1, "Should return exactly one character.");
			TextPart::Character(self.text.as_bytes()[start_idx])
		})
	}
}
