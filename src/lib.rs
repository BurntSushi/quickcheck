#![crate_name = "quickcheck"]
#![doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")]
#![experimental]

//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).
//!
//! For detailed examples, please see the
//! [README](https://github.com/BurntSushi/quickcheck).

#![feature(macro_rules, phase)]

extern crate collections;
#[phase(plugin, link)] extern crate log;

pub use arbitrary::{
    Arbitrary, Gen, StdGen, Shrinker,
    empty_shrinker, single_shrinker,
};
pub use tester::{QuickCheck, Testable, TestResult, quickcheck};

mod arbitrary;
mod tester;

#[cfg(test)]
mod tests;
