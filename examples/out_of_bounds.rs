#![allow(unstable)]

extern crate quickcheck;

use std::iter::range;
use quickcheck::{TestResult, quickcheck};

fn main() {
    fn prop(length: usize, index: usize) -> TestResult {
        let v: Vec<_> = range(0, length).collect();

        if index < length {
            TestResult::discard()
        } else {
            TestResult::must_fail(move || {
                v[index]
            })
        }
    }
    quickcheck(prop as fn(usize, usize) -> TestResult);
}
