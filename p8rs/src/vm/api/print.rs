use core::alloc::Allocator;
use core::pin::Pin;
use std::ops::Not;
use gc_arena::Collect;
use p8rs_piccolo::{BoxSequence, Callback, CallbackReturn, Context, Error, Execution, RuntimeError, RuntimeRef, Sequence, SequencePoll, Stack, String, Value};
use p8rs_types::p8num::P8Num;
use p8rs_types::p8scii::Display;
use crate::vm::font::Font;
use crate::vm::memory::machine_state::{MiscChipsetFeatureFlags, PrintDefaultsFlags};
use crate::vm::memory::painter::CallbackResult;
use crate::vm::Runtime;

pub fn install_pico8_print<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("print", Callback::from_fn(&ctx, |ctx, _exec, mut stack, rt| {
		let rt = rt.downcast::<Runtime>();
		let (text, mut x, y, mut col): (Value, Option<P8Num>, Option<P8Num>, Option<P8Num>) = stack.consume(ctx).unwrap();
		if y.is_none() {
			col = x;
			x = None;
		}
		
		if let Some((x, y)) = x.zip(y) {
			rt.set_cursor_home(x.to_integer());
			rt.set_cursor_position([x.to_integer(), y.to_integer()]);
		}
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
			let (line_height, font_height) = get_line_height(rt, flags);
			println!("print({}) - cursor: {:?}", text, rt.get_cursor_position());
			println!("print: scrolling in case line would not fit, line_height={}, font_height={}", line_height, font_height);
			handle_newline(rt, NewlineRequest::MakeSpaceBeforePrint, flags, y.is_some());
		}else{
			println!("print({}, {}, {}) - cursor: {:?}", text, x.unwrap(), y.unwrap(), rt.get_cursor_position());
		}
		
		Ok(CallbackReturn::Sequence(BoxSequence::new(ctx.mutation(), PrintSeq {
			skip_frames: 0,
			letter_frame_skip: 0,
			text: TextEscapeIterator::new(text),
			next_char: None,
			stopped: false,
			flags: flags.bits(),
			x_wrapped: false,
			y_provided: y.is_some(),
			clipping: false,
			drawn: false,
			max_x: None,
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
	ModifyFlags(PrintDefaultsFlags),
}

fn screen_shift_up_exact(rt: &mut Runtime, shift: u8) {
	println!("screen_shift_up_exact: scrolling, shift={}", shift);
	if rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL) {
		println!("screen_shift_up_exact: jk actually NO_PRINT_SCROLL");
		return;
	}
	
	rt.set_cursor_y(rt.get_cursor_position()[1].overflowing_sub(shift as i16).0);
	rt.memory.screen().shift_up(shift, 0);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NewlineRequest {
	MakeSpaceBeforePrint,
	WrappingNewline,
	ContentNewline,
	PrintEndNewline
}

fn handle_newline(rt: &mut Runtime, request: NewlineRequest, flags: PrintDefaultsFlags, y_passed: bool) {
	println!("handle_newline({:?})", request);
	
	let (line_height, font_height) = get_line_height(rt, flags);
	
	let (reset_x, align_shift, advance_y, considered_height) = match request {
		NewlineRequest::MakeSpaceBeforePrint => (false, false,           0, line_height),
		NewlineRequest::WrappingNewline =>      (true,  false, line_height, line_height),
		NewlineRequest::ContentNewline =>       (true,  true,  line_height, line_height),
		NewlineRequest::PrintEndNewline =>      (true,  true,  line_height, font_height),
	};
	
	if reset_x {
		rt.set_cursor_x(rt.get_cursor_home());
	}
	
	let curr_y = rt.get_cursor_position()[1];
	let new_y = curr_y.overflowing_add(advance_y as i16).0;
	
	rt.set_cursor_y(new_y);
	
	if !y_passed && !rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL) && new_y >= 0 && new_y < 256 {
		let new_y = new_y as u8;
		let max_y = 128 - considered_height;
		if new_y > max_y {
			let mut shift = new_y - max_y;
			if align_shift && shift < font_height {
				shift = font_height;
			}
			println!("handle_newline - shifting - new_y: {}, max_y: {}, shift: {}", new_y, max_y, shift);
			screen_shift_up_exact(rt, shift);
		}
	}
}

fn escape_to_print_flags(val: u8) -> Option<PrintDefaultsFlags> {
	match val {
		b'b' => Some(PrintDefaultsFlags::PADDING),
		b'w' => Some(PrintDefaultsFlags::WIDE),
		b't' => Some(PrintDefaultsFlags::TALL),
		b'#' => Some(PrintDefaultsFlags::SOLID_BG),
		b'i' => Some(PrintDefaultsFlags::INVERT),
		b'=' => Some(PrintDefaultsFlags::DOTTY),
		b'p' => Some(PrintDefaultsFlags::WIDE | PrintDefaultsFlags::TALL | PrintDefaultsFlags::DOTTY),
		_ => None
	}
}

fn execute_escape_sequence<'gc>(_ctx: Context<'gc>, rt: &mut Runtime, flags: PrintDefaultsFlags, bytes: &[u8], y_passed: bool) -> Result<EscapeSequenceAction, RuntimeError<'gc>> {
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
				b'-' => {
					if let Some(flag) = escape_to_print_flags(bytes[2]) {
						println!("Escape: ^- {}", Display(bytes));
						EscapeSequenceAction::ModifyFlags(flags & flag.not())
					}else{
						println!("Escape UNPARSED: {}", Display(bytes));
						debug!("unparsed sequence {:?}", bytes);
						EscapeSequenceAction::Nop
					}
				},
				_ => {
					if let Some(flag) = escape_to_print_flags(arg) {
						println!("Escape parsed: ^ {}", Display(bytes));
						return Ok(EscapeSequenceAction::ModifyFlags(flags | flag | PrintDefaultsFlags::ENABLE))
					}
					debug!("Unimplemented escape sequence! {:?}", bytes);
					EscapeSequenceAction::Nop
				}
			}
		},
		8 => { // \b
			let x = rt.get_cursor_position()[0];
			let font_width = get_font(rt, flags).width();
			rt.set_cursor_x(x.overflowing_sub(font_width as i16).0);
			EscapeSequenceAction::Nop
		},
		9 => { // \t
			let tab_stop = 16; // TODO: check if configurable
			let x = rt.get_cursor_position()[0];
			rt.set_cursor_x((x / tab_stop + 1) * tab_stop);
			EscapeSequenceAction::Nop
		},
		10 => { // \n
			handle_newline(rt, NewlineRequest::ContentNewline, flags, y_passed);
			EscapeSequenceAction::Nop
		},
		13 => { // \r
			rt.set_cursor_x(rt.get_cursor_home());
			EscapeSequenceAction::Nop
		},
		_ => {
			debug!("Unimplemented escape sequence! {:?}", bytes);
			EscapeSequenceAction::Nop
		}
	})
}

/// Returns: (line_height, font_height)
fn get_line_height(rt: &mut Runtime, flags: PrintDefaultsFlags) -> (u8, u8) {
	let font_height = get_font(rt, flags).height();
	if flags.contains(PrintDefaultsFlags::TALL) && flags.contains(PrintDefaultsFlags::ENABLE) {
		(font_height * 2, font_height)
	} else {
		(font_height, font_height)
	}
}


fn get_font(rt: &mut Runtime, flags: PrintDefaultsFlags) -> Font<'_> {
	let use_custom_font = flags.contains(PrintDefaultsFlags::CUSTOM_FONT);
	if use_custom_font {
		Font::new(rt.memory.const_slice(0x5600))
	} else {
		Font::SYSTEM
	}
}

fn draw_letter(_ctx: Context, rt: &mut Runtime, flags: PrintDefaultsFlags, letter: u8, y_passed: bool, dry_run: bool) -> (i16, i16, bool) {
	let is_wide = flags.contains(PrintDefaultsFlags::WIDE);
	let is_tall = flags.contains(PrintDefaultsFlags::TALL);
	let is_inverted = flags.contains(PrintDefaultsFlags::INVERT);
	let is_dotty = flags.contains(PrintDefaultsFlags::DOTTY);
	let is_dotty_x = is_dotty && is_wide && !is_tall;
	let is_dotty_y = is_dotty && is_tall;
	
	let font = get_font(rt, flags);
	
	let font_width = font.width_chr(letter);
	let font_height = font.height();
	let char_font = &font.char(letter);
	let x_stride = if is_wide { 2 } else { 1 };
	let y_stride = if is_tall { 2 } else { 1 };
	let draw_width = font_width * x_stride;
	let draw_height = font_height * y_stride;
	
	let x_wrapped = rt.get_cursor_position()[0].overflowing_add(font_width as i16).0 > 128
		&& rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::PRINT_WRAP);
	
	if x_wrapped {
		handle_newline(rt, NewlineRequest::WrappingNewline, flags, y_passed);
	}
	
	assert!(font_width <= 8, "Char width cannot be >8");
	assert!(font_height <= 8, "Char height cannot be >8");
	
	if !dry_run {
		let [cursor_x, cursor_y] = rt.get_cursor_position();
		let (abs_cursor_x, abs_cursor_y) = rt.memory.painter().to_abs(cursor_x, cursor_y);

		let mut painter = rt.memory.painter().with_callback(|x: u8, y: u8| {
			let local_x = x.overflowing_sub(abs_cursor_x as u8).0;
			let local_y = y.overflowing_sub(abs_cursor_y as u8).0;
			let font_x = if is_wide { local_x / 2 } else { local_x };
			let font_y = if is_tall { local_y / 2 } else { local_y };
			let font_line = char_font[font_y as usize];
			let font_bit = (font_line >> font_x) & 1 != 0;
			
			if (font_bit && !(is_dotty_x && local_x % 2 == 0) && !(is_dotty_y && local_y % 2 == 1)) != is_inverted {
				CallbackResult::Keep
			}else{
				CallbackResult::Discard
			}
		});
		
		painter.set_fill(None).paint(
			cursor_x..cursor_x.saturating_add((font_width*x_stride) as i16),
			cursor_y..cursor_y.saturating_add((font_height*y_stride) as i16)
		);
		
		// for y in 0..font_height {
		// 	let mut font_line = char_font[y as usize];
		// 	for x in 0..font_width {
		// 		let font_bit = font_line & 1 != 0;
		// 		font_line >>= 1;
		//
		// 		for dy in 0..y_stride {
		// 			for dx in 0..x_stride {
		// 				if (font_bit && !(is_dotty_x && dx == 0) && !(is_dotty_y && dy == 1)) != is_inverted {
		// 					let pixel_x = cursor_x.overflowing_add((x * x_stride + dx) as i16).0;
		// 					let pixel_y = cursor_y.overflowing_add((y * y_stride + dy) as i16).0;
		// 					// painter.paint(pixel_x, pixel_y);
		// 				}
		// 			}
		// 		}
		//
		// 	}
		// }
	}
	
	(draw_width as i16, draw_height as i16, x_wrapped)
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
	/// has ever drawn (without clipping)
	drawn: bool,
	/// was y-coord provided when calling print()
	y_provided: bool,
	/// maximum cursor x during printing - for return
	max_x: Option<i16>,
}

impl<'gc> Sequence<'gc> for PrintSeq<'gc> {
	fn poll(
		mut self: Pin<&mut Self>,
		ctx: Context<'gc>,
		_exec: Execution<'gc, '_>,
		mut stack: Stack<'gc, '_>,
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
				let [cursor_x, cursor_y] = rt.get_cursor_position();

				if (cursor_y >= 128 && chipset_flags.contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL))
				|| (cursor_x >= 128 && !chipset_flags.contains(MiscChipsetFeatureFlags::PRINT_WRAP)) {
					if self.drawn { self.clipping = true; }
				} else {
					self.drawn = true;
				}
				
				let (w, _, x_wrapped) = draw_letter(ctx, rt, PrintDefaultsFlags::from_bits_truncate(self.flags), char, self.y_provided, self.clipping);
				
				self.x_wrapped = x_wrapped;
				
				let x = rt.get_cursor_position()[0].overflowing_add(w).0;
				rt.set_cursor_x(x);
				self.max_x = self.max_x.max(Some(x));
			}
			
			let part = self.text.next();
			
			if let Some(part) = part {
				match part {
					TextPart::Character(char) => {
						self.next_char = Some(char);
						self.skip_frames = self.letter_frame_skip;
					}
					TextPart::EscapeSequence(bytes) => {
						match execute_escape_sequence(ctx, rt, PrintDefaultsFlags::from_bits_truncate(self.flags), bytes, self.y_provided)? {
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
			handle_newline(rt, NewlineRequest::PrintEndNewline, PrintDefaultsFlags::from_bits_truncate(self.flags), self.y_provided);
		}
		
		stack.replace(ctx, P8Num::from(self.max_x.unwrap_or(0)));
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
