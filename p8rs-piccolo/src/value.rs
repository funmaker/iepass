use alloc::string::ToString;
use core::fmt;

use gc_arena::{Collect, Gc};
use p8rs_types::p8num::P8Num;
use p8rs_types::p8scii;
use crate::{Callback, Closure, Constant, Function, String, Table, Thread, UserData};

/// The single data type for all Lua variables.
///
/// Every value that Lua code can manipulate directly is ultimately a some kind of `Value`.
#[derive(Debug, Copy, Clone, Collect)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[collect(no_drop)]
pub enum Value<'gc> {
    Nil,
    Boolean(bool),
    Number(P8Num),
    String(#[cfg_attr(feature = "defmt", defmt(Debug2Format))] String<'gc>), // TODO: Format for piccolo Strings?
    Table(Table<'gc>),
    Function(Function<'gc>),
    Thread(Thread<'gc>),
    UserData(UserData<'gc>),
}

impl<'gc> Default for Value<'gc> {
    fn default() -> Self {
        Value::Nil
    }
}

impl<'gc> Value<'gc> {
    pub fn type_name(self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::Thread(_) => "thread",
            Value::UserData(_) => "userdata",
        }
    }

    /// Returns a proxy object which can display any `Value`.
    ///
    /// [`Value::Nil`] is printed as "nil", booleans, integers, and numbers are always printed as
    /// directly as they would be from Rust.
    ///
    /// [`Value::String`] is printed using the [`String::display_lossy`] method, which displays
    /// strings in a lossy fashion if they are not UTF-8 internally.
    ///
    /// [`Value::Table`]s, [`Value::Function`]s, [`Value::Thread`]s, and [`Value::UserData`]
    /// are all printed as `"<typename {:p}>"`, where 'typename' is the value returned by
    /// [`Value::type_name`].
    pub fn display(self) -> ValueDisplay<'gc> {
        ValueDisplay(self)
    }

    pub(crate) fn debug_shallow(self) -> impl fmt::Debug + 'gc {
        struct ShallowDebug<'gc>(Value<'gc>);

        impl<'gc> fmt::Debug for ShallowDebug<'gc> {
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
                    Value::Table(t) => {
                        write!(fmt, "Value::Table({:p})", Gc::as_ptr(t.into_inner()))
                    }
                    Value::Function(Function::Closure(c)) => {
                        write!(
                            fmt,
                            "Value::Function(Function::Closure({:p}))",
                            Gc::as_ptr(c.into_inner())
                        )
                    }
                    Value::Function(Function::Callback(c)) => {
                        write!(
                            fmt,
                            "Value::Function(Function::Callback({:p}))",
                            Gc::as_ptr(c.into_inner())
                        )
                    }
                    Value::Thread(t) => {
                        write!(fmt, "Value::Thread({:p})", Gc::as_ptr(t.into_inner()))
                    }
                    Value::UserData(u) => {
                        write!(fmt, "Value::UserData({:p})", Gc::as_ptr(u.into_inner()))
                    }
                    v => write!(fmt, "{:?}", v),
                }
            }
        }

        ShallowDebug(self)
    }

    pub fn is_nil(self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Lua `nil` and `false` are false, anything else is true.
    pub fn to_bool(self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(false) => false,
            _ => true,
        }
    }

    /// Interprets Numbers and Strings as a Number, if possible.
    pub fn to_number(self) -> Option<P8Num> {
        self.to_constant().and_then(|c| c.to_number())
    }

    /// Interprets Numbers and Strings as a String, otherwise returns None.
    ///
    /// If the value is a [`Value::String`], the string is returned directly. Otherwise, the
    /// returned string will always be the same as what [`Value::display`] would display.
    pub fn into_string(self, ctx: crate::Context<'gc>) -> Option<String<'gc>> {
        match self {
            Value::Number(n) => Some(ctx.intern(n.to_string().as_bytes())),
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Indicates whether the value can be implicitly converted to a [`String`]; if so,
    /// [`Value::into_string`] will always return `Some`.
    pub fn is_implicit_string(self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn to_constant(self) -> Option<Constant<String<'gc>>> {
        match self {
            Value::Nil => Some(Constant::Nil),
            Value::Boolean(b) => Some(Constant::Boolean(b)),
            Value::Number(n) => Some(Constant::Number(n)),
            Value::String(s) => Some(Constant::String(s)),
            _ => None,
        }
    }
}

impl<'gc> From<bool> for Value<'gc> {
    fn from(v: bool) -> Value<'gc> {
        Value::Boolean(v)
    }
}

impl<'gc> From<P8Num> for Value<'gc> {
    fn from(v: P8Num) -> Value<'gc> {
        Value::Number(v)
    }
}

impl<'gc> From<u8> for Value<'gc> {
    fn from(v: u8) -> Value<'gc> {
        Value::Number(P8Num::from(v))
    }
}

impl<'gc> From<i8> for Value<'gc> {
    fn from(v: i8) -> Value<'gc> {
        Value::Number(P8Num::from(v))
    }
}

impl<'gc> From<i16> for Value<'gc> {
    fn from(v: i16) -> Value<'gc> {
        Value::Number(P8Num::from(v))
    }
}

impl<'gc> From<f64> for Value<'gc> {
    fn from(v: f64) -> Value<'gc> {
        Value::Number(P8Num::new_f64(v))
    }
}

impl<'gc, S> From<Constant<S>> for Value<'gc>
where
    String<'gc>: From<S>,
{
    fn from(constant: Constant<S>) -> Self {
        match constant {
            Constant::Nil => Value::Nil,
            Constant::Boolean(b) => Value::Boolean(b),
            Constant::Number(n) => Value::Number(n),
            Constant::String(s) => Value::String(s.into()),
        }
    }
}

impl<'gc> From<String<'gc>> for Value<'gc> {
    fn from(v: String<'gc>) -> Value<'gc> {
        Value::String(v)
    }
}

impl<'gc> From<Table<'gc>> for Value<'gc> {
    fn from(v: Table<'gc>) -> Value<'gc> {
        Value::Table(v)
    }
}

impl<'gc> From<Function<'gc>> for Value<'gc> {
    fn from(v: Function<'gc>) -> Value<'gc> {
        Value::Function(v)
    }
}

impl<'gc> From<Closure<'gc>> for Value<'gc> {
    fn from(v: Closure<'gc>) -> Value<'gc> {
        Value::Function(Function::Closure(v))
    }
}

impl<'gc> From<Callback<'gc>> for Value<'gc> {
    fn from(v: Callback<'gc>) -> Value<'gc> {
        Value::Function(Function::Callback(v))
    }
}

impl<'gc> From<Thread<'gc>> for Value<'gc> {
    fn from(v: Thread<'gc>) -> Value<'gc> {
        Value::Thread(v)
    }
}

impl<'gc> From<UserData<'gc>> for Value<'gc> {
    fn from(v: UserData<'gc>) -> Value<'gc> {
        Value::UserData(v)
    }
}

pub struct ValueDisplay<'gc>(Value<'gc>);

impl<'gc> fmt::Display for ValueDisplay<'gc> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Value::Nil => write!(fmt, "nil"),
            Value::Boolean(b) => write!(fmt, "{}", b),
            Value::Number(f) => write!(fmt, "{}", f),
            Value::String(s) => {
                for char in p8scii::to_iter(s.as_bytes()) {
                    write!(fmt, "{}", char)?;
                }
                Ok(())
            },
            Value::Table(t) => write!(fmt, "<table {:p}>", Gc::as_ptr(t.into_inner())),
            Value::Function(Function::Closure(c)) => {
                write!(fmt, "<function {:p}>", Gc::as_ptr(c.into_inner()))
            }
            Value::Function(Function::Callback(c)) => {
                write!(fmt, "<function {:p}>", Gc::as_ptr(c.into_inner()))
            }
            Value::Thread(t) => write!(fmt, "<thread {:p}>", Gc::as_ptr(t.into_inner())),
            Value::UserData(u) => {
                write!(fmt, "<userdata {:p}>", Gc::as_ptr(u.into_inner()))
            }
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ValueDisplay<'_> {
    fn format(&self, fmt: defmt::Formatter) {
        match self.0 {
            Value::Nil => defmt::write!(fmt, "nil"),
            Value::Boolean(b) => defmt::write!(fmt, "{}", b),
            Value::Number(f) => defmt::write!(fmt, "{}", f),
            Value::String(s) => {
                for char in p8scii::to_iter(s.as_bytes()) {
                    defmt::write!(fmt, "{}", char);
                }
            },
            Value::Table(t) => defmt::write!(fmt, "<table 0x{:x}>", Gc::as_ptr(t.into_inner()) as usize),
            Value::Function(Function::Closure(c)) => {
                defmt::write!(fmt, "<function 0x{:x}>", Gc::as_ptr(c.into_inner()) as usize)
            },
            Value::Function(Function::Callback(c)) => {
                defmt::write!(fmt, "<function 0x{:x}>", Gc::as_ptr(c.into_inner()) as usize)
            },
            Value::Thread(t) => {
                defmt::write!(fmt, "<thread 0x{:x}>", Gc::as_ptr(t.into_inner()) as usize)
            },
            Value::UserData(u) => {
                defmt::write!(fmt, "<userdata 0x{:x}>", Gc::as_ptr(u.into_inner()) as usize)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use gc_arena::Rootable;
    
    use crate::table::Table;
    use crate::{Lua, UserData};

    #[test]
    fn recursive_table_debug() {
        let mut lua = Lua::core();
        lua.enter(|ctx| {
            let table = Table::new(&ctx);
            table.set_field(ctx, "a", table);
            println!("{:?}", table);
        
            let table2 = Table::new(&ctx);
            table2.set_metatable(&ctx, Some(table2));
            println!("{:?}", table2);
        
            let combined = Table::new(&ctx);
            combined.set_field(ctx, "a", combined);
            combined.set_metatable(&ctx, Some(combined));
            println!("{:?}", combined);
        
            let user = UserData::new::<Rootable![()]>(&ctx, ());
            user.set_metatable(&ctx, Some(combined));
            println!("{:?}", user);
        });
    }
}
