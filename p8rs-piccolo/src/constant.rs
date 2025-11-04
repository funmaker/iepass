use core::hash::{Hash, Hasher};

use gc_arena::Collect;
use p8rs_types::p8num::P8Num;
use crate::compiler::string_utils::trim_whitespace;

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum Constant<S> {
    Nil,
    Boolean(bool),
    Number(P8Num),
    String(S),
}

impl<S> Constant<S> {
    pub fn to_bool(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(false) => false,
            _ => true,
        }
    }

    pub fn not(&self) -> Constant<S> {
        Constant::Boolean(!self.to_bool())
    }

    pub fn as_string_ref(&self) -> Constant<&S> {
        match self {
            Constant::Nil => Constant::Nil,
            Constant::Boolean(b) => Constant::Boolean(*b),
            Constant::Number(n) => Constant::Number(*n),
            Constant::String(s) => Constant::String(s),
        }
    }

    pub fn map_string<S2>(self, f: impl FnOnce(S) -> S2) -> Constant<S2> {
        match self {
            Constant::Nil => Constant::Nil,
            Constant::Boolean(b) => Constant::Boolean(b),
            Constant::Number(n) => Constant::Number(n),
            Constant::String(s) => Constant::String(f(s)),
        }
    }
}

impl<S: AsRef<[u8]>> Constant<S> {
    /// Interprets Numbers and Strings as a Number, if possible.
    pub fn to_number(&self) -> Option<P8Num> {
        match self {
            &Self::Number(a) => Some(a),
            Self::String(a) => {
                let a = trim_whitespace(a.as_ref());
                if let Ok(n) = P8Num::from_ascii(a) {
                    Some(n)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // Mathematical operators

    pub fn add(&self, rhs: &Self) -> Option<Self> {
        Some(match (self, rhs) {
            (a, b) => Self::Number(a.to_number()? + b.to_number()?),
        })
    }

    pub fn subtract(&self, rhs: &Self) -> Option<Self> {
        Some(match (self, rhs) {
            (a, b) => Self::Number(a.to_number()? - b.to_number()?),
        })
    }

    pub fn multiply(&self, rhs: &Self) -> Option<Self> {
        Some(match (self, rhs) {
            (a, b) => Self::Number(a.to_number()? * b.to_number()?),
        })
    }

    /// This operation always returns a Number, even when called with Integer arguments.
    pub fn float_divide(&self, rhs: &Self) -> Option<Self> {
        Some(Self::Number(self.to_number()? / rhs.to_number()?))
    }

    /// This operation returns an Integer only if both arguments are Integers. Rounding is towards
    /// negative infinity.
    pub fn floor_divide(&self, rhs: &Self) -> Option<Self> {
        match (self, rhs) {
            (a, b) => {
                let a = a.to_number()?;
                let b = b.to_number()?;
                Some(Self::Number((a / b).floor()))
            }
        }
    }

    /// Computes the Lua modulus (`%`) operator. This is unlike Rust's `%` operator which computes
    /// the remainder.
    pub fn modulo(&self, rhs: &Self) -> Option<Self> {
        match (self, rhs) {
            (a, b) => {
                let (a, b) = (a.to_number()?, b.to_number()?);
                Some(Self::Number(((a % b) + b) % b))
            }
        }
    }

    /// This operation always returns a Number, even when called with Integer arguments.
    pub fn exponentiate(&self, rhs: &Self) -> Option<Self> {
        let lhs = self.to_number()?;
        let rhs = rhs.to_number()?;
        Some(Self::Number(lhs.powf(rhs).unwrap_or(P8Num::ZERO)))
    }

    pub fn negate(&self) -> Option<Self> {
        match self {
            &Self::Number(a) => Some(Self::Number(-a)),
            s => s.to_number().map(|x| Self::Number(-x)),
        }
    }

    // Bitwise operators

    pub fn bitwise_not(&self) -> Option<Self> {
        Some(Self::Number(!self.to_number()?))
    }

    pub fn bitwise_and(&self, rhs: &Self) -> Option<Self> {
        Some(Self::Number(self.to_number()? & rhs.to_number()?))
    }

    pub fn bitwise_or(&self, rhs: &Self) -> Option<Self> {
        Some(Self::Number(self.to_number()? | rhs.to_number()?))
    }

    pub fn bitwise_xor(&self, rhs: &Self) -> Option<Self> {
        Some(Self::Number(self.to_number()? ^ rhs.to_number()?))
    }
    
    pub fn shift_right_arithmetic(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.to_number()?.floor();
        if rhs < P8Num::ZERO {
            return self.shift_left(&Self::Number(-rhs));
        }
        let rhs = i32::from(rhs) as u32;
        Some(Self::Number(self.to_number()?.checked_shr(rhs).unwrap_or(P8Num::ZERO)))
    }
    
    pub fn shift_right_logical(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.to_number()?.floor();
        if rhs < P8Num::ZERO {
            return self.shift_left(&Self::Number(-rhs));
        }
        let rhs = i32::from(rhs) as u32;
        Some(Self::Number(self.to_number()?
                              .to_raw()
                              .cast_unsigned()
                              .checked_shr(rhs)
                              .map_or(P8Num::ZERO, |raw| P8Num::from_raw(raw as i32))))
    }

    pub fn shift_left(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.to_number()?.floor();
        if rhs < P8Num::ZERO {
            return self.shift_right_arithmetic(&Self::Number(-rhs));
        }
        let rhs = i32::from(rhs) as u32;
        Some(Self::Number(self.to_number()?.checked_shl(rhs).unwrap_or(P8Num::ZERO)))
    }
    
    pub fn rotate_right(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.to_number()?.floor();
        if rhs < P8Num::ZERO {
            return self.rotate_left(&Self::Number(-rhs));
        }
        let rhs = i32::from(rhs) as u32;
        Some(Self::Number(self.to_number()?.rotate_right(rhs)))
    }
    
    pub fn rotate_left(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.to_number()?.floor();
        if rhs < P8Num::ZERO {
            return self.rotate_right(&Self::Number(-rhs));
        }
        let rhs = i32::from(rhs) as u32;
        Some(Self::Number(self.to_number()?.rotate_left(rhs)))
    }

    // Comparison operators

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Nil, _) => false,

            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Boolean(_), _) => false,

            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Number(_), _) => false,

            (Self::String(a), Self::String(b)) => a.as_ref() == b.as_ref(),
            (Self::String(_), _) => false,
        }
    }

    pub fn less_than(&self, rhs: &Self) -> Option<bool> {
        Some(match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => a < b,
            (Self::String(a), Self::String(b)) => a.as_ref() < b.as_ref(),
            _ => return None,
        })
    }

    pub fn less_equal(&self, rhs: &Self) -> Option<bool> {
        Some(match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => a <= b,
            (Self::String(a), Self::String(b)) => a.as_ref() <= b.as_ref(),
            _ => return None,
        })
    }
}

impl<S: AsRef<[u8]>> PartialEq for Constant<S> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

/// Wrapper for a `Constant` that implements Hash and Eq, and only compares equal when the types are
/// bit for bit identical.
#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct IdenticalConstant<S>(pub Constant<S>);

impl<S> From<Constant<S>> for IdenticalConstant<S> {
    fn from(value: Constant<S>) -> Self {
        Self(value)
    }
}

impl<S: AsRef<[u8]>> PartialEq for IdenticalConstant<S> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Constant::Nil, Constant::Nil) => true,
            (Constant::Nil, _) => false,

            (Constant::Boolean(a), Constant::Boolean(b)) => a == b,
            (Constant::Boolean(_), _) => false,

            (Constant::Number(a), Constant::Number(b)) => a == b,
            (Constant::Number(_), _) => false,

            (Constant::String(a), Constant::String(b)) => a.as_ref() == b.as_ref(),
            (Constant::String(_), _) => false,
        }
    }
}

impl<S: AsRef<[u8]>> Eq for IdenticalConstant<S> {}

impl<S: AsRef<[u8]>> Hash for IdenticalConstant<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            Constant::Nil => {
                Hash::hash(&0, state);
            }
            Constant::Boolean(b) => {
                Hash::hash(&1, state);
                b.hash(state);
            }
            Constant::Number(n) => {
                Hash::hash(&3, state);
                n.hash(state);
            }
            Constant::String(s) => {
                Hash::hash(&4, state);
                s.as_ref().hash(state);
            }
        }
    }
}
