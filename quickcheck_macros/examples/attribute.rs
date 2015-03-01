#![feature(plugin)]
#![allow(dead_code)]
#![plugin(quickcheck_macros)]

extern crate quickcheck;

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs {
        rev.insert(0, x.clone())
    }
    rev
}

#[quickcheck]
fn double_reversal_is_identity(xs: Vec<isize>) -> bool {
    xs == reverse(&reverse(&xs))
}

fn main() {}
