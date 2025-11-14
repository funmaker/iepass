//! Formatting macros for p8rs.
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(unused_macros)]

pub use cfg_if::cfg_if;

cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::assert;
    } else {
        pub use core::assert;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::assert_eq;
    } else {
        pub use core::assert_eq;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::assert_ne;
    } else {
        pub use core::assert_ne;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::debug_assert;
    } else {
        pub use core::debug_assert;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::debug_assert_eq;
    } else {
        pub use core::debug_assert_eq;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::debug_assert_ne;
    } else {
        pub use core::debug_assert_ne;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::unreachable;
    } else {
        pub use core::unreachable;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::todo;
    } else {
        pub use core::todo;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::unimplemented;
    } else {
        pub use core::unimplemented;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::panic;
    } else {
        pub use core::panic;
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::trace;
    } else if #[cfg(feature = "log-04")] {
        pub use log_04::trace;
    } else if #[cfg(all(feature = "std", feature = "std-trace"))] {
        pub use std::println as trace;
    } else {
        #[macro_export] macro_rules! trace { ($($x: tt)*) => {let _ = ($( & $x ),*);} }
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::debug;
    } else if #[cfg(feature = "log-04")] {
        pub use log_04::debug;
    } else if #[cfg(all(feature = "std", feature = "std-debug"))] {
        pub use std::println as debug;
    } else {
        #[macro_export] macro_rules! debug { ($($x: tt)*) => {let _ = ($( & $x ),*);} }
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::info;
    } else if #[cfg(feature = "log-04")] {
        pub use log_04::info;
    } else if #[cfg(all(feature = "std", feature = "std-info"))] {
        pub use std::println as info;
    } else {
        #[macro_export] macro_rules! info { ($($x: tt)*) => {let _ = ($( & $x ),*);} }
    }
}



cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::warn;
    } else if #[cfg(feature = "log-04")] {
        pub use log_04::warn;
    } else if #[cfg(all(feature = "std", feature = "std-warn"))] {
        pub use std::println as warn;
    } else {
        #[macro_export] macro_rules! warn { ($($x: tt)*) => {let _ = ($( & $x ),*);} }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::error;
    } else if #[cfg(feature = "log-04")] {
        pub use log_04::error;
    } else if #[cfg(all(feature = "std", feature = "std-error"))] {
        pub use std::eprintln as error;
    } else {
        #[macro_export] macro_rules! error { ($($x: tt)*) => {let _ = ($( & $x ),*);} }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "defmt")] {
        pub use defmt::println;
    } else if #[cfg(feature = "log-04")] {
        pub use log_04::error as println;
    } else if #[cfg(all(feature = "std", feature = "std-error"))] {
        pub use std::println;
    } else {
        #[macro_export] macro_rules! println { ($($x: tt)*) => {let _ = ($( & $x ),*);} }
    }
}

#[macro_export]
macro_rules! print {
    ($($x: tt)*) => {
        compile_error!("print! macro is disallowed")
    };
}
