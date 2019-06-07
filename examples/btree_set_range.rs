extern crate quickcheck;

use quickcheck::{quickcheck, TestResult};
use std::collections::BTreeSet;
use std::ops::{Bound, RangeBounds};

/// Covers every `std::ops::Range*` plus variants with exclusive start.
type RangeAny<T> = (Bound<T>, Bound<T>);

/// Checks conditions where `BTreeSet::range` panics:
/// - Panics if range start > end.
/// - Panics if range start == end and both bounds are Excluded.
fn panics(range: RangeAny<i32>) -> bool {
    use Bound::{Excluded as Ex, Included as In, Unbounded};
    match range {
        (Ex(s), Ex(e)) => s >= e,
        (In(s), Ex(e)) | (Ex(s), In(e)) | (In(s), In(e)) => s > e,
        (Unbounded, _) | (_, Unbounded) => false,
    }
}

/// Checks that `BTreeSet::range` returns all items contained in the given `range`.
fn check_range(set: BTreeSet<i32>, range: RangeAny<i32>) -> TestResult {
    if panics(range) {
        TestResult::discard()
    } else {
        let xs: BTreeSet<_> = set.range(range).cloned().collect();
        TestResult::from_bool(set.iter().all(|x| range.contains(x) == xs.contains(x)))
    }
}

fn main() {
    quickcheck(check_range as fn(_, _) -> TestResult);
}
