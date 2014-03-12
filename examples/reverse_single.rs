extern crate quickcheck;

use quickcheck::{TestResult, quickcheck};

fn reverse<T: Clone>(xs: &[T]) -> ~[T] {
    let mut rev = ~[];
    for x in xs.iter() {
        rev.unshift(x.clone())
    }
    rev
}

fn main() {
    fn prop(xs: ~[int]) -> ~TestResult {
        if xs.len() != 1 {
            return TestResult::discard()
        }
        TestResult::from_bool(xs == reverse(xs))
    }
    quickcheck(prop);
}
