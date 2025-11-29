use crate::vm::memory::Memory;

pub trait PainterCallback {
	fn check(&mut self, memory: &mut Memory, x: u8, y: u8) -> CallbackResult;
}

pub struct Noop;
impl PainterCallback for Noop {
	#[inline(always)]
	fn check(&mut self, _: &mut Memory, _: u8, _: u8) -> CallbackResult {
		CallbackResult::Keep
	}
}

impl<T, R> PainterCallback for T
where T: Fn(&mut Memory, u8, u8) -> R,
      R: Into<CallbackResult> {
	#[inline(always)]
	fn check(&mut self, memory: &mut Memory, x: u8, y: u8) -> CallbackResult {
		self(memory, x, y).into()
	}
}

pub enum CallbackResult {
	Discard,
	Keep,
	Color(u8),
}

impl From<bool> for CallbackResult {
	fn from(set: bool) -> CallbackResult {
		if set {
			CallbackResult::Keep
		} else {
			CallbackResult::Discard
		}
	}
}

impl From<u8> for CallbackResult {
	fn from(col: u8) -> CallbackResult {
		CallbackResult::Color(col)
	}
}

impl From<Option<u8>> for CallbackResult {
	fn from(col: Option<u8>) -> CallbackResult {
		if let Some(col) = col {
			CallbackResult::Color(col)
		} else {
			CallbackResult::Discard
		}
	}
}
