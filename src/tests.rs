use std::cmp::Ord;
use std::iter;
use std::num::Float;
use super::{QuickCheck, TestResult, quickcheck};

#[test]
fn prop_oob() {
    fn prop() -> bool {
        let zero: Vec<bool> = vec![];
        zero[0]
    }
    match QuickCheck::new().quicktest(prop) {
        Ok(n) => panic!("prop_oob should fail with a runtime error \
                        but instead it passed {} tests.", n),
        _ => return,
    }
}

#[test]
fn prop_reverse_reverse() {
    fn prop(xs: Vec<uint>) -> bool {
        let rev: Vec<uint> = xs.clone().into_iter().rev().collect();
        let revrev = rev.into_iter().rev().collect();
        xs == revrev
    }
    quickcheck(prop);
}

#[test]
fn reverse_single() {
    fn prop(xs: Vec<uint>) -> TestResult {
        if xs.len() != 1 {
            return TestResult::discard()
        }
        return TestResult::from_bool(
            xs == xs.clone().into_iter().rev().collect()
        )
    }
    quickcheck(prop);
}

#[test]
fn reverse_app() {
    fn prop(xs: Vec<uint>, ys: Vec<uint>) -> bool {
        let mut app = xs.clone();
        app.push_all(ys.as_slice());
        let app_rev: Vec<uint> = app.into_iter().rev().collect();

        let rxs: Vec<uint> = xs.into_iter().rev().collect();
        let mut rev_app = ys.into_iter().rev().collect::<Vec<uint>>();
        rev_app.extend(rxs.into_iter());

        app_rev == rev_app
    }
    quickcheck(prop);
}

#[test]
fn max() {
    fn prop(x: int, y: int) -> TestResult {
        if x > y {
            return TestResult::discard()
        } else {
            return TestResult::from_bool(::std::cmp::max(x, y) == y)
        }
    }
    quickcheck(prop);
}

#[test]
fn sort() {
    fn prop(mut xs: Vec<int>) -> bool {
        xs.sort_by(|x, y| x.cmp(y));
        let upto = if xs.len() == 0 { 0 } else { xs.len()-1 };
        for i in iter::range(0, upto) {
            if xs[i] > xs[i+1] {
                return false
            }
        }
        true
    }
    quickcheck(prop);
}

#[test]
#[should_fail]
fn sieve_of_eratosthenes() {
    fn sieve(n: uint) -> Vec<uint> {
        if n <= 1 {
            return vec![];
        }

        let mut marked = Vec::from_fn(n+1, |_| false);
        marked[0] = true;
        marked[1] = true;
        marked[2] = false;
        for p in iter::range(2, n) {
            for i in iter::range_step(2 * p, n, p) { // whoops!
                marked[i] = true;
            }
        }
        let mut primes = vec![];
        for (i, &m) in marked.iter().enumerate() {
            if !m { primes.push(i) }
        }
        primes
    }

    fn prop(n: uint) -> bool {
        let primes = sieve(n);
        primes.iter().all(|&i| is_prime(i))
    }
    fn is_prime(n: uint) -> bool {
        if n == 0 || n == 1 {
            return false;
        } else if n == 2 {
            return true;
        }

        let max_possible = (n as f64).sqrt().ceil() as uint;
        for i in iter::range_inclusive(2, max_possible) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }
    quickcheck(prop);
}
