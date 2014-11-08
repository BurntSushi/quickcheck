#![crate_name = "quickcheck"]
#![crate_type = "lib"]
#![license = "UNLICENSE"]
#![doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")]

//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).
//!
//! For detailed examples, please see the
//! [README](https://github.com/BurntSushi/quickcheck).

#![feature(macro_rules, phase)]

extern crate collections;
#[phase(plugin, link)] extern crate log;

pub use arbitrary::{
    Arbitrary, Gen, StdGen, Shrinker, gen,
    empty_shrinker, single_shrinker,
};
pub use tester::{
    Testable, TestResult, Config,
    quickcheck, quickcheck_config, quicktest, quicktest_config,
    DEFAULT_CONFIG, DEFAULT_SIZE,
};

mod arbitrary;
mod tester;

#[cfg(test)]
mod tests;
