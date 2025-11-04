mod executor;
mod thread;
mod vm;

use gc_arena::Collect;
use thiserror::Error;
use alloc::vec::Vec;
use core::fmt::Write;
use crate::compiler::LineNumber;
use crate::meta_ops::{MetaCallError, MetaOperatorError};
use crate::RuntimeError;
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
pub struct TracebackEntry<'gc> {
    pub name: Option<crate::String<'gc>>,
    pub line_number: LineNumber,
}

impl<'gc> TracebackEntry<'gc> {
    pub fn to_extern(&self) -> ExternTracebackEntry {
        ExternTracebackEntry::from(self)
    }
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct ExternTracebackEntry {
    pub name: Option<alloc::string::String>,
    pub line_number: LineNumber,
}

impl ExternTracebackEntry {
    pub fn write<'gc>(&self, target: & mut impl Write) -> Result<(), RuntimeError> {
        // offset line numbers to match pico 8
        write!(target, "  [string \"-- pico-8 header...\"]:{}: in ", self.line_number.0 + 2)?;
        if let Some(name) = &self.name {
            write!(target, "function '{}'", name)?;
        }else{
            write!(target, "main chunk")?;
        }
        write!(target, "\n")?;
        Ok(())
    }
}

impl<'gc> From<&TracebackEntry<'gc>> for ExternTracebackEntry {
    fn from(value: &TracebackEntry<'gc>) -> Self {
        Self {
            name: value.name.map(|name| name.to_string()),
            line_number: value.line_number,
        }
    }
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct Traceback {
    pub entries: Vec<ExternTracebackEntry>,
}

impl<'gc> Traceback {
    pub fn new(entries: impl Iterator<Item=&'gc TracebackEntry<'gc>>) -> Self {
        Self { entries: entries.map(Into::into).collect() }
    }
    
    pub(crate) fn empty() -> Self {
        Self { entries: Vec::new() }
    }
    
    pub(crate) fn add_entry(&mut self, entry: &TracebackEntry<'gc>) {
        self.entries.push(entry.into());
    }
}
