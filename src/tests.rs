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
    match QuickCheck::new().quicktest(prop as fn() -> bool) {
        Ok(n) => panic!("prop_oob should fail with a runtime error \
                         but instead it passed {} tests.", n),
        _ => return,
    }
}

#[test]
fn prop_reverse_reverse() {
    fn prop(xs: Vec<usize>) -> bool {
        let rev: Vec<_> = xs.clone().into_iter().rev().collect();
        let revrev: Vec<_> = rev.into_iter().rev().collect();
        xs == revrev
    }
    quickcheck(prop as fn(Vec<usize>) -> bool);
}

#[test]
fn reverse_single() {
    fn prop(xs: Vec<usize>) -> TestResult {
        if xs.len() != 1 {
            return TestResult::discard()
        }
        return TestResult::from_bool(
            xs == xs.clone().into_iter().rev().collect::<Vec<_>>()
        )
    }
    quickcheck(prop as fn(Vec<usize>) -> TestResult);
}

#[test]
fn reverse_app() {
    fn prop(xs: Vec<usize>, ys: Vec<usize>) -> bool {
        let mut app = xs.clone();
        app.extend(ys.iter().cloned());
        let app_rev: Vec<usize> = app.into_iter().rev().collect();

        let rxs: Vec<usize> = xs.into_iter().rev().collect();
        let mut rev_app = ys.into_iter().rev().collect::<Vec<usize>>();
        rev_app.extend(rxs.into_iter());

        app_rev == rev_app
    }
    quickcheck(prop as fn(Vec<usize>, Vec<usize>) -> bool);
}

#[test]
fn max() {
    fn prop(x: isize, y: isize) -> TestResult {
        if x > y {
            TestResult::discard()
        } else {
            TestResult::from_bool(::std::cmp::max(x, y) == y)
        }
    }
    quickcheck(prop as fn(isize, isize) -> TestResult);
}

#[test]
fn sort() {
    fn prop(mut xs: Vec<isize>) -> bool {
        xs.sort_by(|x, y| x.cmp(y));
        let upto = if xs.len() == 0 { 0 } else { xs.len()-1 };
        for i in 0..upto {
            if xs[i] > xs[i+1] {
                return false
            }
        }
        true
    }
    quickcheck(prop as fn(Vec<isize>) -> bool);
}

#[test]
#[should_fail]
fn sieve_of_eratosthenes() {
    fn sieve(n: usize) -> Vec<usize> {
        if n <= 1 {
            return vec![];
        }

        let mut marked: Vec<_> = (0..n+1).map(|_| false).collect();
        marked[0] = true;
        marked[1] = true;
        marked[2] = false;
        for p in 2..n {
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

    fn prop(n: usize) -> bool {
        let primes = sieve(n);
        primes.iter().all(|&i| is_prime(i))
    }
    fn is_prime(n: usize) -> bool {
        if n == 0 || n == 1 {
            return false;
        } else if n == 2 {
            return true;
        }

        let max_possible = (n as f64).sqrt().ceil() as usize;
        for i in iter::range_inclusive(2, max_possible) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }
    quickcheck(prop as fn(usize) -> bool);
}
