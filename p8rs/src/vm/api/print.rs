use core::pin::Pin;
use core::ops::Not;
use gc_arena::Collect;
use p8rs_piccolo::{BoxSequence, Callback, CallbackReturn, Context, Error, Execution, RuntimeError, RuntimeRef, Sequence, SequencePoll, Stack, String, Value};
use p8rs_types::p8num::P8Num;

use crate::vm::font::Font;
use crate::vm::memory::machine_state::{MiscChipsetFeatureFlags, PrintDefaultsFlags};
use crate::vm::memory::Memory;
use crate::vm::memory::painter::CallbackResult;
use crate::vm::Runtime;

pub fn load(ctx: Context) {
	ctx.set_global("print", Callback::from_fn(&ctx, |ctx, _exec, mut stack, rt| {
		let rt = rt.downcast::<Runtime>();
		let (text, mut x, y, mut col): (Option<Value>, Option<P8Num>, Option<P8Num>, Option<P8Num>) = stack.consume(ctx).unwrap();
		if y.is_none() {
			col = x;
			x = None;
		}
		
		if let Some((x, y)) = x.zip(y) {
			rt.set_cursor_home(x.to_integer());
			rt.set_cursor_position([x.to_integer(), y.to_integer()]);
		}
		if let Some(col) = col { rt.memory.machine_state().set_pen_color(col); }
		
		let text = super::base::tostr(ctx, text, None);
		
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
		
		let flags_enabled = flags.contains(PrintDefaultsFlags::ENABLE);
		let state = PrintState {
			advance_x: 0,
			advance_x_wide: 0,
			advance_y: 0,
			background_color: None,
			padding:     flags_enabled && flags.contains(PrintDefaultsFlags::PADDING),
			wide:        flags_enabled && flags.contains(PrintDefaultsFlags::WIDE),
			tall:        flags_enabled && flags.contains(PrintDefaultsFlags::TALL),
			solid_bg:    flags_enabled && flags.contains(PrintDefaultsFlags::SOLID_BG),
			invert:      flags_enabled && flags.contains(PrintDefaultsFlags::INVERT),
			dotty:       flags_enabled && flags.contains(PrintDefaultsFlags::DOTTY),
			custom_font: flags_enabled && flags.contains(PrintDefaultsFlags::CUSTOM_FONT),
			pinball: false,
			wrap: rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::PRINT_WRAP),
		};
		
		if y.is_none() {
			handle_newline(rt, NewlineRequest::MakeSpaceBeforePrint, &state, y.is_some());
		}
		
		Ok(CallbackReturn::Sequence(BoxSequence::new(ctx.mutation(), PrintSeq {
			skip_frames: 0,
			letter_frame_skip: 0,
			text: TextEscapeIterator::new(text),
			next_char: None,
			stopped: false,
			state,
			x_wrapped: false,
			y_provided: y.is_some(),
			clipping: false,
			drawn: false,
			max_pos: None,
		})))
	}));
}

fn control_arg_to_number(arg: u8) -> Option<u8> {
	Some(match arg {
		b'0'..=b'9' => arg - b'0',
		b'a'.. => arg - b'a' + 0xa,
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
	ModifyState(PrintState),
}

fn screen_shift_up_exact(rt: &mut Runtime, shift: u8) {
	if rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL) {
		return;
	}
	
	rt.set_cursor_y(rt.get_cursor_position()[1].overflowing_sub(shift as i16).0);
	rt.memory.screen().shift_up(&mut rt.memory, shift, 0);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NewlineRequest {
	MakeSpaceBeforePrint,
	WrappingNewline,
	ContentNewline,
	PrintEndNewline
}

fn handle_newline(rt: &mut Runtime, request: NewlineRequest, state: &PrintState, y_passed: bool) {
	let (line_height, font_height) = get_line_height(rt, state);
	
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
			screen_shift_up_exact(rt, shift);
		}
	}
}

fn apply_escape_flag_to_print_state(val: u8, state: &mut PrintState, value: bool) -> bool {
	match val {
		b'b' => state.padding = value,
		b'w' => state.wide = value,
		b't' => state.tall = value,
		b'#' => state.solid_bg = value,
		b'i' => state.invert = value,
		b'=' => state.dotty = value,
		b'p' => state.pinball = value,
		b'$' => state.wrap = value,
		_ => return false
	}
	true
}

fn execute_escape_sequence<'gc>(_ctx: Context<'gc>, rt: &mut Runtime, state: &mut PrintState, bytes: &[u8], y_passed: bool) -> Result<EscapeSequenceAction, RuntimeError<'gc>> {
	assert!(bytes.len() > 0, "Escape sequence must not be empty");
	
	let escape_code = bytes[0];
	Ok(match escape_code {
		0 => EscapeSequenceAction::Stop,
		2 => { // \#
			state.background_color = control_arg_to_number(bytes[1]);
			EscapeSequenceAction::Nop
		},
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
					if apply_escape_flag_to_print_state(bytes[2], state, false) {
						// EscapeSequenceAction::ModifyState(flags & flag.not())
						EscapeSequenceAction::Nop
					}else{
						debug!("unparsed sequence {:?}", bytes);
						EscapeSequenceAction::Nop
					}
				},
				_ => {
					if apply_escape_flag_to_print_state(arg, state, true) {
						// return Ok(EscapeSequenceAction::ModifyState(flags | flag | PrintDefaultsFlags::ENABLE))
						return Ok(EscapeSequenceAction::Nop)
					}
					debug!("Unimplemented escape sequence! {:?}", bytes);
					EscapeSequenceAction::Nop
				}
			}
		},
		8 => { // \b
			let x = rt.get_cursor_position()[0];
			let font_width = get_font(rt, state).width();
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
			handle_newline(rt, NewlineRequest::ContentNewline, state, y_passed);
			EscapeSequenceAction::Nop
		},
		12 => { // \f
			if let Some(arg) = control_arg_to_number(bytes[1]) {
				*rt.memory.machine_state().pen_color() &= 0xfu8.not();
				*rt.memory.machine_state().pen_color() |= arg & 0xfu8;
				// TODO: check if top is zeroed
			}
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
fn get_line_height(rt: &mut Runtime, state: &PrintState) -> (u8, u8) {
	let font_height = get_font(rt, state).height();
	if state.tall || state.pinball {
		(font_height * 2, font_height)
	} else {
		(font_height, font_height)
	}
}

fn get_font<'a>(rt: &'a mut Runtime, state: &PrintState) -> Font<'a> {
	if state.custom_font {
		Font::new(rt.memory.const_slice_mut(0x5600))
	} else {
		Font::SYSTEM
	}
}

fn draw_letter(_ctx: Context, rt: &mut Runtime, state: &PrintState, letter: u8, y_passed: bool, dry_run: bool) -> (i16, i16, bool) {
	let is_wide = state.pinball || state.wide;
	let is_tall = state.pinball || state.tall;
	let is_inverted = state.invert;
	let is_solid = state.solid_bg;
	let is_dotty = state.dotty;
	let is_dotty_x = state.pinball || is_dotty && is_wide && !is_tall;
	let is_dotty_y = state.pinball || is_dotty && is_tall;
	
	let bg = state.background_color.or(is_solid.then_some((*rt.memory.machine_state().pen_color() & 0xf0) >> 4));
	
	let font = get_font(rt, state);
	
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
		handle_newline(rt, NewlineRequest::WrappingNewline, state, y_passed);
	}
	
	assert!(font_width <= 8, "Char width cannot be >8");
	assert!(font_height <= 8, "Char height cannot be >8");
	
	if !dry_run {
		let [cursor_x, cursor_y] = rt.get_cursor_position();
		let (abs_cursor_x, abs_cursor_y) = rt.memory.painter().to_abs(cursor_x, cursor_y);
		
		rt.memory
		  .painter()
		  .text_mode(&mut rt.memory, bg)
		  .paint_tex(
			  &mut rt.memory,
			  cursor_x..cursor_x.saturating_add((font_width*x_stride) as i16),
			  cursor_y..cursor_y.saturating_add((font_height*y_stride) as i16),
			  |_: &mut Memory, x: u8, y: u8| {
				  let local_x = x.overflowing_sub(abs_cursor_x as u8).0;
				  let local_y = y.overflowing_sub(abs_cursor_y as u8).0;
				  let font_x = if is_wide { local_x / 2 } else { local_x };
				  let font_y = if is_tall { local_y / 2 } else { local_y };
				  let font_line = char_font[font_y as usize];
				  let font_bit = (font_line >> font_x) & 1 != 0;
				  
				  if (font_bit && !(is_dotty_x && local_x % 2 == 0) && !(is_dotty_y && local_y % 2 == 1)) != is_inverted {
					  CallbackResult::Keep
				  }else if bg.is_some() {
					  CallbackResult::Color(0)
				  } else {
					  CallbackResult::Discard
				  }
			  }
		  );
	}
	
	(draw_width as i16, draw_height as i16, x_wrapped)
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
struct PrintState {
	pub advance_x: u8,
	pub advance_x_wide: u8,
	pub advance_y: u8,
	pub background_color: Option<u8>, // \# P0
	pub padding: bool,
	pub wide: bool,
	pub tall: bool,
	pub solid_bg: bool,
	pub invert: bool,
	pub dotty: bool,
	pub custom_font: bool,
	pub pinball: bool,
	pub wrap: bool,
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
	state: PrintState,
	/// has x wrapped (only if character wrapping is enabled)
	x_wrapped: bool,
	/// should print stop drawing because we ran out either x-space (without character wrapping) or y-space (without line scrolling)
	clipping: bool,
	/// has ever drawn (without clipping)
	drawn: bool,
	/// was y-coord provided when calling print()
	y_provided: bool,
	/// maximum cursor pos during printing - for return
	max_pos: Option<(i16, i16)>,
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
		
		while !self.stopped {
			if self.skip_frames > 0 {
				self.skip_frames -= 1;
				return Ok(SequencePoll::Yield { to_thread: None, bottom: 0 })
			}
			
			if let Some(char) = self.next_char {
				self.next_char = None;
				
				let chipset_flags = *rt.memory.machine_state().misc_chipset_flags();
				let [cursor_x, cursor_y] = rt.get_cursor_position();

				if (cursor_y >= 128 && chipset_flags.contains(MiscChipsetFeatureFlags::NO_PRINT_SCROLL))
				|| (cursor_x >= 128 && !self.state.wrap) {
					if self.drawn { self.clipping = true; }
				} else {
					self.drawn = true;
				}
				
				let (w, h, x_wrapped) = draw_letter(ctx, rt, &self.state, char, self.y_provided, self.clipping);
				
				self.x_wrapped = x_wrapped;
				
				let [x, y] = rt.get_cursor_position();
				let x = x.wrapping_add(w);
				let y = y.wrapping_add(h);
				rt.set_cursor_x(x);
				if let Some((old_x, old_y)) = self.max_pos {
					self.max_pos = Some((old_x.max(x), old_y.max(y)));
				} else {
					self.max_pos = Some((x, y));
				}
			}
			
			let Some(part) = self.text.next() else { break };
			
			match part {
				TextPart::Character(char) => {
					self.next_char = Some(char);
					self.skip_frames = self.letter_frame_skip;
				}
				TextPart::EscapeSequence(bytes) => {
					let y_provided = self.y_provided;
					match execute_escape_sequence(ctx, rt, &mut self.state, bytes, y_provided)? {
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
						EscapeSequenceAction::ModifyState(new_state) => {
							self.state = new_state;
						}
					}
				}
				TextPart::UnterminatedEscapeSequence(_) => debug!("[PrintSeq] called on a sequence containing UnterminatedEscapeSequence!"),
			}
		
		}
		
		if !self.stopped && !rt.memory.machine_state().misc_chipset_flags().contains(MiscChipsetFeatureFlags::NO_PRINT_NEWLINE) {
			handle_newline(rt, NewlineRequest::PrintEndNewline, &self.state, self.y_provided);
		}
		
		
		let [_, cursor_y] = rt.get_cursor_position();
		stack.replace(ctx, self.max_pos.unwrap_or((0, 0)));
		
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
