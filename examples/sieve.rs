#![allow(unstable)]

extern crate quickcheck;

use std::iter;
use std::num::Float;

use quickcheck::quickcheck;

fn sieve(n: usize) -> Vec<usize> {
    if n <= 1 {
        return vec!()
    }

    let mut marked: Vec<_> = iter::range(0, n+1).map(|_| false).collect();
    marked[0] = true;
    marked[1] = true;
    marked[2] = false;
    for p in iter::range(2, n) {
        for i in iter::range_step(2 * p, n, p) { // whoops!
            marked[i] = true;
        }
    }
    let mut primes = vec!();
    for (i, &m) in marked.iter().enumerate() {
        if !m { primes.push(i) }
    }
    primes
}

fn is_prime(n: usize) -> bool {
    if n == 0 || n == 1 {
        return false
    } else if n == 2 {
        return true
    }

    let max_possible = (n as f64).sqrt().ceil() as usize;
    for i in iter::range_inclusive(2, max_possible) {
        if n % i == 0 {
            return false
        }
    }
    return true
}

fn prop_all_prime(n: usize) -> bool {
    let primes = sieve(n);
    primes.iter().all(|&i| is_prime(i))
}

fn main() {
    quickcheck(prop_all_prime as fn(usize) -> bool);
}
