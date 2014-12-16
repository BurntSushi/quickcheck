extern crate quickcheck;

use quickcheck::{TestResult, quickcheck};

fn main() {
    fn prop(length: uint, index: uint) -> TestResult {
        let v = Vec::from_fn(length, |i| i);

        if index < length {
            TestResult::discard()
        } else {
            TestResult::must_fail(move || {
                v[index]
            })
        }
    }
    quickcheck(prop);
}
