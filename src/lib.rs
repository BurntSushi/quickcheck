// I have no idea what I'm doing with these attributes. Are we using
// semantic versioning? Some packages include their full github URL.
// Documentation for this stuff is extremely scarce.
#[crate_id = "quickcheck#0.1.0"];
#[crate_type = "lib"];
#[license = "UNLICENSE"];
#[doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")];

//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).

extern crate collections;

pub use arbitrary::{Arbitrary, Gen, StdGen, arbitrary, default_gen, gen};

pub use shrink::{ObjIter, Shrink};

mod arbitrary;
mod shrink;

