use crate::pico8::api::gfx::{draw_letter, set_cursor_color};
use crate::pico8::memory::PrintAttributeFlags;
use crate::pico8::Runtime;
use core::alloc::Allocator;
use gc_arena::Collect;
use p8rs_piccolo::{BoxSequence, Callback, CallbackReturn, Context, Error, Execution, RuntimeError, RuntimeRef, Sequence, SequencePoll, Stack, String, Value};
use std::pin::Pin;

pub fn install_pico8_print<A: Allocator + 'static>(ctx: Context) {
	ctx.set_global("printx", Callback::from_fn(&ctx, |ctx, _exec, mut stack, rt| {
		let rt = rt.downcast::<Runtime>();
		let (text, x, y, color): (Value, Option<i16>, Option<i16>, Option<i16>) = stack.consume(ctx).unwrap();
		set_cursor_color(&mut rt.memory.draw_state(), x, y, color);
		
		let text = if let Value::String(text) = text {
			text
		} else {
			if let Value::String(str) = super::base::tostr(ctx, text, None).unwrap().unwrap() {
				str
			}else{
				debug!("[print] Could not convert value to string when printing!");
				String::from_slice(ctx.mutation(), "")
			}
		};
		
		let last_part = TextEscapeIterator::new(text).last();
		if last_part.is_none() {
			// todo: return val
			return Ok(CallbackReturn::Return);
		}
		
		let last_part = last_part.unwrap();
		if matches!(last_part, TextPart::UnterminatedEscapeSequence(_)) {
			debug!("[print()] String contained unterminated escape sequence, discarding.");
			// todo: return val
			return Ok(CallbackReturn::Return);
		}
		
		let flags = rt.memory.hardware_state().get_print_defaults();
		
		Ok(CallbackReturn::Sequence(BoxSequence::new(ctx.mutation(), PrintSeq {
			skip_frames: 0,
			letter_frame_skip: 0,
			text: TextEscapeIterator::new(text),
			next_char: None,
			stopped: false,
			flags: flags.bits(),
		})))
	}));
}

fn control_arg_to_number(arg: u8) -> Option<u8> {
	Some(match arg {
		b'0'..=b'9' => arg - b'0',
		b'a'.. => arg - b'a',
		_ => {
			debug!("Unexpected control code argmument: {}", arg);
			return None;
		}
	})
}

enum EscapeSequenceAction {
	Nop,
	Stop,
	SkipFrames(usize),
	SetLetterFrameSkip(usize),
	ModifyFlags(PrintAttributeFlags), // todo
}

fn execute_escape_sequence<'gc, A: Allocator>(_ctx: Context<'gc>, _rt: &mut Runtime<A>, bytes: &[u8]) -> Result<EscapeSequenceAction, RuntimeError> {
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
					let arg = control_arg_to_number(bytes[2]).unwrap(); // todo safety
					EscapeSequenceAction::SetLetterFrameSkip(arg as usize)
				}
				_ => {
					debug!("Unimplemented escape sequence! {:?}", bytes);
					EscapeSequenceAction::Nop
				}
			}
		}
		_ => {
			debug!("Unimplemented escape sequence! {:?}", bytes);
			EscapeSequenceAction::Nop
		}
	})
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
	flags: u8
}

impl<'gc> Sequence<'gc> for PrintSeq<'gc> {
	fn poll(
		mut self: Pin<&mut Self>,
		ctx: Context<'gc>,
		_exec: Execution<'gc, '_>,
		_stack: Stack<'gc, '_>,
		rt: RuntimeRef<'_>,
	) -> Result<SequencePoll<'gc>, Error<'gc>> {
		if !self.stopped {
			if self.skip_frames > 0 {
				self.skip_frames -= 1;
				return Ok(SequencePoll::Yield { to_thread: None, bottom: 0 })
			}
			
			let rt = rt.downcast::<Runtime>();
			
			if let Some(char) = self.next_char {
				self.next_char = None;
				let mut cursor_x = rt.memory.draw_state().cursor_position()[0] as i16;
				if cursor_x < 128 {
					// todo: verify pico-8 behaviour / add line wrapping
					cursor_x += draw_letter(rt, PrintAttributeFlags::from_bits_truncate(self.flags), char)?;
				}
				rt.memory.draw_state().cursor_position()[0] = cursor_x.min(255) as u8;
			}
			
			let part = self.text.next();
			
			if let Some(part) = part {
				match part {
					TextPart::Character(char) => {
						self.next_char = Some(char);
						self.skip_frames = self.letter_frame_skip;
					}
					TextPart::EscapeSequence(bytes) => {
						match execute_escape_sequence(ctx, rt, bytes)? {
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
		
		// no next part
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
