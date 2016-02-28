//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).
//!
//! For detailed examples, please see the
//! [README](https://github.com/BurntSushi/quickcheck).

#![allow(deprecated)] // for connect -> join in 1.3
#![cfg_attr(feature = "unstable", feature(core_intrinsics, time2))]

extern crate env_logger;
#[macro_use] extern crate log;
extern crate rand;

pub use arbitrary::{
    Arbitrary, Gen, StdGen, RAnd, ROr, Restricted, Restriction,
    empty_shrinker, single_shrinker,
};
pub use rand::Rng;
pub use tester::{QuickCheck, Testable, TestResult, quickcheck};

mod arbitrary;
mod tester;

#[cfg(test)]
mod tests;
