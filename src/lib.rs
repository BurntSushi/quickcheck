// I have no idea what I'm doing with these attributes. Are we using
// semantic versioning? Some packages include their full github URL.
// Documentation for this stuff is extremely scarce.
#[crate_id = "quickcheck#0.1.0"];
#[crate_type = "lib"];
#[license = "UNLICENSE"];
#[warn(experimental)];

extern crate collections;

pub use shrink::{Shrink};

mod shrink;

