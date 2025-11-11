use core::fmt::{Display, Write};
use alloc::string::String;
use anyhow::anyhow;
use p8rs_piccolo::compiler::LineNumber;
use p8rs_piccolo::RuntimeError;
use p8rs_piccolo::thread::{ExternTracebackEntry, TracebackEntry};

pub(crate) trait PrintableTracebackEntry {
	fn with_name<F, R>(&self, f: F) -> R
	where
		F: FnOnce(Option<&str>) -> R;
	
	fn line_number(&self) -> LineNumber;
}

impl<'gc> PrintableTracebackEntry for TracebackEntry<'gc> {
	fn with_name<F, R>(&self, f: F) -> R
	where
		F: FnOnce(Option<&str>) -> R
	{
		let name = self.name.as_ref().map(|s| s.to_string());
		f(name.as_ref().map(|s| s.as_str()))
	}
	
	fn line_number(&self) -> LineNumber {
		self.line_number
	}
}

impl<'gc> PrintableTracebackEntry for ExternTracebackEntry {
	fn with_name<F, R>(&self, f: F) -> R
	where
		F: FnOnce(Option<&str>) -> R
	{
		f(self.name.as_ref().map(|s| s.as_str()))
	}
	
	fn line_number(&self) -> LineNumber {
		self.line_number
	}
}


pub(crate) fn write_traceback_entries<'gc, 'a, T>(target: & mut impl Write, entries: impl Iterator<Item=&'a T>) -> Result<usize, RuntimeError>
	where
		T: PrintableTracebackEntry + 'a {
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
