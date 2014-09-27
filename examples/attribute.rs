#![feature(phase)]
#![allow(dead_code)]

extern crate quickcheck;
#[phase(plugin)]
extern crate quickcheck_macros;

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs.iter() {
        rev.insert(0, x.clone())
    }
    rev
}

#[quickcheck]
fn double_reversal_is_identity(xs: Vec<int>) -> bool {
    xs == reverse(reverse(xs.as_slice()).as_slice())
}

fn main() {}
