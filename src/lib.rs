//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).
//!
//! For detailed examples, please see the
//! [README](https://github.com/BurntSushi/quickcheck).

#![allow(deprecated)] // for connect -> join in 1.3

extern crate env_logger;
#[macro_use] extern crate log;
extern crate num;
extern crate rand;

pub use arbitrary::{
    Arbitrary, Gen, StdGen, TestResult
};
pub use rand::Rng;
pub use tester::{QuickCheck, quickcheck};

mod shrink;
mod arbitrary;
mod entropy_pool;
mod tester;

#[cfg(test)]
mod tests;
