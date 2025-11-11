mod executor;
mod thread;
mod vm;

use gc_arena::{Collect, DynamicRootSet, Mutation};
use thiserror::Error;
use alloc::vec::Vec;
use core::fmt::Write;
use crate::compiler::LineNumber;
use crate::meta_ops::{MetaCallError, MetaOperatorError};
use crate::{Context, RuntimeError, StashedString};
use crate::stash::{Fetchable, Stashable};
pub use self::{
    executor::{
        BadExecutorMode, CurrentThread, Execution, Executor, ExecutorInner, ExecutorMode,
        UpperLuaFrame,
    },
    thread::{BadThreadMode, OpenUpValue, Thread, ThreadInner, ThreadMode},
};

#[derive(Debug, Clone, Error)]
pub enum VMError {
    #[error("{}", if *.0 {
        "operation expects variable stack"
    } else {
        "unexpected variable stack during operation"
    })]
    ExpectedVariableStack(bool),
    #[error("Bad types for SetList op, expected table, integer, found {0}, {1}")]
    BadSetList(&'static str, &'static str),
    #[error("bad call")]
    BadCall(#[from] MetaCallError),
    #[error("operator error")]
    OperatorError(#[from] MetaOperatorError),
    #[error("_ENV upvalue is only allowed on top-level closure")]
    BadEnvUpValue,
    #[error("Invalid types in for loop; expected numbers, found {0}, {1}, and {2}")]
    BadForLoop(&'static str, &'static str, &'static str),
    #[error("Invalid types in for loop; expected numbers, found {0} and {1}")]
    BadForLoopPrep(&'static str, &'static str),
}


#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct TracebackEntryBase<S> {
    pub name: Option<S>,
    pub line_number: LineNumber,
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct TracebackBase<S> {
    pub entries: Vec<TracebackEntryBase<S>>,
}

pub type StashedTracebackEntry = TracebackEntryBase<StashedString>;
pub type TracebackEntry<'gc> = TracebackEntryBase<crate::String<'gc>>;
pub type ExternTracebackEntry = TracebackEntryBase<alloc::string::String>;

pub type StashedTraceback = TracebackBase<StashedString>;
pub type Traceback<'gc> = TracebackBase<crate::String<'gc>>;
pub type ExternTraceback = TracebackBase<alloc::string::String>;

impl<S> TracebackBase<S>
{
    fn into_traceback<D>(self) -> TracebackBase<D>
    where TracebackEntryBase<S>: Into<TracebackEntryBase<D>>,{
        TracebackBase {
            entries: self.entries.into_iter().map(|e| Into::into(e)).collect()
        }
    }
}

impl<'gc> From<TracebackEntry<'gc>> for ExternTracebackEntry {
    fn from(value: TracebackEntry<'gc>) -> Self {
        ExternTracebackEntry {
            line_number: value.line_number,
            name: value.name.map(|s| s.to_string())
        }
    }
}



impl<S> TracebackBase<S> {
    pub(crate) fn empty() -> Self {
        Self { entries: Vec::new() }
    }
    
    pub(crate) fn add_entry(&mut self, entry: TracebackEntryBase<S>) {
        self.entries.push(entry);
    }
}

impl Fetchable for StashedTracebackEntry {
    type Fetched<'gc> = TracebackEntry<'gc>;
    
    fn fetch<'gc>(&self, roots: DynamicRootSet<'gc>) -> Self::Fetched<'gc> {
        TracebackEntry {
            line_number: self.line_number,
            name: self.name.as_ref().map(|s| s.fetch(roots))
        }
    }
}

impl<'gc> Stashable<'gc> for TracebackEntry<'gc> {
    type Stashed = StashedTracebackEntry;
    
    fn stash(self, mc: &Mutation<'gc>, roots: DynamicRootSet<'gc>) -> Self::Stashed {
        StashedTracebackEntry {
            line_number: self.line_number,
            name: self.name.as_ref().map(|s| s.stash(mc, roots))
        }
    }
}

impl Fetchable for StashedTraceback {
    type Fetched<'gc> = Traceback<'gc>;
    
    fn fetch<'gc>(&self, roots: DynamicRootSet<'gc>) -> Self::Fetched<'gc> {
        Traceback {
            entries: self.entries.iter().map(|entry| entry.fetch(roots)).collect()
        }
    }
}

impl<'gc> Stashable<'gc> for Traceback<'gc> {
    type Stashed = StashedTraceback;
    
    fn stash(self, mc: &Mutation<'gc>, roots: DynamicRootSet<'gc>) -> Self::Stashed {
        StashedTraceback {
            entries: self.entries.into_iter().map(|entry| entry.stash(mc, roots)).collect()
        }
    }
}


