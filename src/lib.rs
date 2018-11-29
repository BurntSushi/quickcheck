//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).
//!
//! For detailed examples, please see the
//! [README](https://github.com/BurntSushi/quickcheck).

#![allow(deprecated)] // for connect -> join in 1.3

#![cfg_attr(feature = "i128", feature(i128_type, i128))]

#[cfg(feature = "use_logging")]
extern crate env_logger;
#[cfg(feature = "use_logging")]
#[macro_use] extern crate log;
extern crate rand;

pub use arbitrary::{
    Arbitrary, Gen, StdGen,
    empty_shrinker, single_shrinker,
};
pub use rand::Rng;
pub use tester::{QuickCheck, Testable, TestResult, quickcheck};

/// A macro for writing quickcheck tests.
///
/// This macro takes as input one or more property functions to test, and
/// produces a proper `#[test]` function for each property. If the property
/// fails, the behavior is as if `quickcheck` were called on the property
/// (i.e., it panics and fails the test).
///
/// Note that this macro doesn't support `mut` or patterns in parameters.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate quickcheck; fn main() {
/// quickcheck! {
///     fn prop_reverse_reverse(xs: Vec<usize>) -> bool {
///         let rev: Vec<_> = xs.clone().into_iter().rev().collect();
///         let revrev: Vec<_> = rev.into_iter().rev().collect();
///         xs == revrev
///     }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! quickcheck {
    (@as_items $($i:item)*) => ($($i)*);
    {
        $(
            $(#[$m:meta])*
            fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) -> $ret:ty {
                $($code:tt)*
            }
        )*
    } => (
        quickcheck! {
            @as_items
            $(
                #[test]
                $(#[$m])*
                fn $fn_name() {
                    fn prop($($arg_name: $arg_ty),*) -> $ret {
                        $($code)*
                    }
                    $crate::quickcheck(prop as fn($($arg_ty),*) -> $ret);
                }
            )*
        }
    )
}

#[cfg(feature = "use_logging")]
fn env_logger_init() -> Result<(), log::SetLoggerError> {
    env_logger::try_init()
}

#[cfg(not(feature = "use_logging"))]
fn env_logger_init() { }
#[cfg(not(feature = "use_logging"))]
macro_rules! info {
    ($($_ignore:tt)*) => { () };
}

#[macro_export]
macro_rules! tuplify {
    () => {
        ()
    };

    ($e:expr) => {
        $e
    };

    ($tuple_item:expr, $($tail:expr),+) => {
        ($tuple_item, tuplify!($($tail),+))
    };
}

#[macro_export]
macro_rules! tuplify_pattern {
    () => {
        ()
    };

    ($p:pat) => {
        $p
    };

    ($tuple_pattern:pat, $($tail:pat),+) => {
        ($tuple_pattern, tuplify_pattern!($($tail),+))
    };
}

#[test]
fn tuplifiy_test() {
    assert_eq!((1, (2, (3, 4))), tuplify!(1,2,3,4));
}

#[test]
fn tuplify_pattern_test() {
    let tuplify_pattern!(a, b, c) = tuplify!(1, 2, 3);
    assert_eq!((a, b, c), (1, 2, 3));
}



mod arbitrary;
mod tester;

#[cfg(test)]
mod tests;
