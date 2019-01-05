#![allow(non_upper_case_globals)]

extern crate quickcheck;
extern crate quickcheck_macros;

use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn min(x: isize, y: isize) -> TestResult {
    if x < y {
        TestResult::discard()
    } else {
        TestResult::from_bool(::std::cmp::min(x, y) == y)
    }
}

#[quickcheck]
#[should_panic]
fn fail_fn() -> bool { false }

#[quickcheck]
static static_bool: bool = true;

#[quickcheck]
#[should_panic]
static fail_static_bool: bool = false;

// If static_bool wasn't turned into a test function, then this should
// result in a compiler error.
#[test]
fn static_bool_test_is_function() {
    static_bool()
}
