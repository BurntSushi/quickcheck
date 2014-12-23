extern crate quickcheck;

use quickcheck::quickcheck;
use std::iter;
use std::num::Float;

fn sieve(n: uint) -> Vec<uint> {
    if n <= 1 {
        return vec!()
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
    let mut primes = vec!();
    for (i, &m) in marked.iter().enumerate() {
        if !m { primes.push(i) }
    }
    primes
}

fn is_prime(n: uint) -> bool {
    if n == 0 || n == 1 {
        return false
    } else if n == 2 {
        return true
    }

    let max_possible = (n as f64).sqrt().ceil() as uint;
    for i in iter::range_inclusive(2, max_possible) {
        if n % i == 0 {
            return false
        }
    }
    return true
}

fn prop_all_prime(n: uint) -> bool {
    let primes = sieve(n);
    primes.iter().all(|&i| is_prime(i))
}

fn main() {
    quickcheck(prop_all_prime as fn(uint) -> bool);
}
