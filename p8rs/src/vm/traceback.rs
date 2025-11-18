use core::fmt::{Display, Write};
use anyhow::Result;
use p8rs_piccolo::traceback::{TracebackEntryBase};

pub(crate) fn write_traceback_entries<'a, S: Display + 'a>(target: &mut impl Write, entries: impl Iterator<Item=&'a TracebackEntryBase<S>>) -> Result<usize> {
	let mut entries = entries.peekable();
	let mut entries_written = 0;
	
	while let Some(entry) = entries.next() {
		let has_next_entry = entries.peek().is_some();
		
		// offset line numbers to match pico 8
		write!(target, "\t[string \"-- pico-8 header...\"]:{}: in ", entry.line_number.0 + 2)?;
		
		if let Some(name) = entry.name.as_ref() {
			write!(target, "function '{}'", name)?;
		} else {
			if has_next_entry {
				write!(target, "anonymous function")?;
			} else {
				write!(target, "main chunk")?;
			}
		}
		
		write!(target, "\n")?;
		
		entries_written += 1;
	}
	
	Ok(entries_written)
}
