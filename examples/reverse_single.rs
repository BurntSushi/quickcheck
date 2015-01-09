extern crate quickcheck;

use quickcheck::{TestResult, quickcheck};

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs.iter() {
        rev.insert(0, x.clone())
    }
    rev
}

fn main() {
    fn prop(xs: Vec<isize>) -> TestResult {
        if xs.len() != 1 {
            return TestResult::discard()
        }
        TestResult::from_bool(xs == reverse(xs.as_slice()))
    }
    quickcheck(prop as fn(Vec<isize>) -> TestResult);
}
