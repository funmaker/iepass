use core::fmt::{Display, Write};
use alloc::string::String;
use anyhow::anyhow;
use p8rs_piccolo::compiler::LineNumber;
use p8rs_piccolo::RuntimeError;
use p8rs_piccolo::thread::ExternTracebackEntry;

pub(crate) trait TracebackEntry<S: Display> {
	fn with_name<F, R>(&self, f: F) -> R
	where F: FnOnce(&Option<S>) -> R;
	
	fn line_number(&self) -> LineNumber;
}

impl TracebackEntry<String> for ExternTracebackEntry {
	fn with_name<F, R>(&self, f: F) -> R
	where F: FnOnce(&Option<String>) -> R
	{
		f(&self.name)
	}
	
	fn line_number(&self) -> LineNumber {
		self.line_number
	}
}

impl<'gc> TracebackEntry<p8rs_piccolo::String<'gc>> for p8rs_piccolo::thread::TracebackEntry<'gc> {

	fn with_name<F, R>(&self, f: F) -> R
	where
		F: FnOnce(&Option<p8rs_piccolo::String<'gc>>) -> R
	{
		f(&self.name)
	}
	
	fn line_number(&self) -> LineNumber {
		self.line_number
	}
}

pub(crate) fn write_traceback_entries<'gc, 'a, S: Display, T>(target: & mut impl Write, entries: impl Iterator<Item=&'a T>) -> Result<usize, RuntimeError>
	where
		T: TracebackEntry<S> + 'a {
	let mut entries = entries.peekable();
	let mut entries_written = 0;
	
	while let Some(entry) = entries.next() {
		let has_next_entry = entries.peek().is_some();
		
		// offset line numbers to match pico 8
		write!(target, "\t[string \"-- pico-8 header...\"]:{}: in ", entry.line_number().0 + 2)?;
		
		entry.with_name(|name| {
			if let Some(name) = name {
				write!(target, "function '{}'", name).ok()
			} else {
				if has_next_entry {
					write!(target, "anonymous function").ok()
				}else{
					write!(target, "main chunk").ok()
				}
			}
		}).ok_or_else(|| anyhow!("Cannot write!"))?;
		
		write!(target, "\n")?;
		
		entries_written += 1;
	}
	
	Ok(entries_written)
}
