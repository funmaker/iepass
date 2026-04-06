#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![feature(type_alias_impl_trait)]

#[macro_use] extern crate p8rs_log;
extern crate alloc;
extern crate core;

pub mod any;
pub mod async_callback;
pub mod callback;
pub mod closure;
pub mod compiler;
pub mod constant;
pub mod conversion;
pub mod error;
pub mod finalizers;
pub mod fuel;
pub mod function;
pub mod lua;
pub mod meta_ops;
pub mod opcode;
pub mod registry;
pub mod stack;
pub mod stash;
pub mod string;
pub mod table;
pub mod thread;
pub mod types;
pub mod userdata;
pub mod value;
pub mod peek_nth;
pub mod runtime;
pub mod traceback;

#[cfg(feature = "std")]
pub mod io;

pub use self::{
    async_callback::{async_sequence, SequenceReturn},
    callback::{BoxSequence, Callback, CallbackFn, CallbackReturn, Sequence, SequencePoll},
    closure::{Closure, CompilerError, FunctionPrototype},
    constant::Constant,
    conversion::{FromMultiValue, FromValue, IntoMultiValue, IntoValue, Variadic},
    error::{Error, ExternError, RuntimeError, TypeError},
    fuel::Fuel,
    function::Function,
    lua::{Context, Lua},
    meta_ops::MetaMethod,
    registry::{Registry, Singleton},
    stack::Stack,
    stash::{
        StashedCallback, StashedClosure, StashedError, StashedExecutor, StashedFunction,
        StashedString, StashedTable, StashedThread, StashedUserData, StashedValue,
        StashedTraceback,
    },
    string::String,
    table::Table,
    thread::{Execution, Executor, ExecutorMode, Thread, ThreadMode},
    userdata::UserData,
    value::Value,
    runtime::{Runtime, RuntimeRef},
    traceback::{Traceback, ExternTraceback}
};
