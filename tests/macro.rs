#![feature(phase)]

#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;

use quickcheck::TestResult;

#[quickcheck]
fn min(x: int, y: int) -> TestResult {
    if x < y {
        return TestResult::discard()
    } else {
        return TestResult::from_bool(::std::cmp::min(x, y) == y)
    }
}

#[quickcheck]
#[should_fail]
fn fail_fn() -> bool { false }

#[quickcheck]
static static_bool: bool = true;

#[quickcheck]
#[should_fail]
static fail_static_bool: bool = false;

// If static_bool wasn't turned into a test function, then this should
// result in a compiler error.
#[test]
fn static_bool_test_is_function() {
    static_bool()
}
