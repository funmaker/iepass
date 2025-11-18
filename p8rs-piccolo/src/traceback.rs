use core::fmt::{Display, Formatter};
use core::cell::Ref;
use alloc::string::String as StdString;
use alloc::vec::Vec;
use gc_arena::{Collect, Gc, RefLock};
use crate::compiler::LineNumber;
use crate::{Context, String};

#[derive(Debug, Clone, Collect)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[collect(no_drop)]
pub struct TracebackEntryBase<S> {
	pub name: Option<S>,
	pub line_number: LineNumber,
}

pub type TracebackEntry<'gc> = TracebackEntryBase<String<'gc>>;
pub type ExternTracebackEntry = TracebackEntryBase<StdString>;

#[derive(Debug, Clone, Collect)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[collect(no_drop)]
pub struct TracebackBase<S> {
	pub entries: Vec<TracebackEntryBase<S>>,
}

pub type TracebackInner<'gc> = RefLock<TracebackBase<String<'gc>>>;

#[derive(Debug, Clone, Collect)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[collect(no_drop)]
pub struct Traceback<'gc>(
	#[cfg_attr(feature = "defmt", defmt(Debug2Format))] // TODO: Implement Format for gc_area
	Gc<'gc, TracebackInner<'gc>>,
);

pub type ExternTraceback = TracebackBase<StdString>;

impl<'gc> Traceback<'gc> {
	pub(crate) fn new(ctx: &Context<'gc>, entries: impl IntoIterator<Item = TracebackEntry<'gc>>) -> Self {
		Self(Gc::new(
			&ctx,
			RefLock::new(TracebackBase {
				entries: entries.into_iter().collect(),
			}),
		))
	}
	
	pub(crate) fn empty(ctx: &Context<'gc>) -> Self {
		Self::new(ctx, [])
	}
	
	pub(crate) fn add_entry(&mut self, ctx: &Context<'gc>, entry: TracebackEntry<'gc>) {
		self.0.borrow_mut(&ctx).entries.push(entry);
	}
	
	pub fn entries(&self) -> Ref<'_, [TracebackEntry<'gc>]> {
		Ref::map(self.0.borrow(), |inner| inner.entries.as_slice())
	}
	
	pub fn from_inner(inner: Gc<'gc, TracebackInner<'gc>>) -> Self {
		Self(inner)
	}
	
	pub fn into_inner(self) -> Gc<'gc, TracebackInner<'gc>> {
		self.0
	}
}

impl<'gc> Traceback<'gc> {
	pub fn to_extern(&self) -> ExternTraceback {
		let entries =
			self.0
				.borrow()
				.entries
			    .iter()
			    .map(|entry| ExternTracebackEntry {
				    name: entry.name.as_ref().map(|s| alloc::format!("{:?}", s)),
				    line_number: entry.line_number,
			    })
			    .collect();
		
		ExternTraceback { entries }
	}
}

impl<S> Display for TracebackBase<S>
where S: Display {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		for entry in self.entries.iter() {
			write!(f, "line {}", entry.line_number)?;
			if let Some(name) = &entry.name {
				write!(f, " in function '{}'", name)?;
			}
			write!(f, "\n")?;
		}
		Ok(())
	}
}
