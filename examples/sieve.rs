extern crate quickcheck;

use std::iter;
use std::vec;
use quickcheck::quickcheck;

fn sieve(n: uint) -> ~[uint] {
    if n <= 1 {
        return ~[]
    }

    let mut marked = vec::from_fn(n+1, |_| false);
    marked[0] = true; marked[1] = true; marked[2] = false;
    let next_marked = |marked: &[bool], p: uint| -> Option<uint> {
        for (i, m) in marked.iter().enumerate() {
            if i > p && !m { return Some(i) }
        }
        None
    };
    let mut next = next_marked(marked, 1);
    while !next.is_none() {
        let p = next.unwrap();
        for i in iter::range_step(2 * p, n, p) {
            marked[i] = true;
        }
        next = next_marked(marked, p);
    }
    iter::range(0, n+1)
        .filter_map(|i| if marked[i] { None } else { Some(i) })
        .to_owned_vec()
}

fn is_prime(n: uint) -> bool {
    if n == 0 || n == 1 {
        return false
    } else if n == 2 {
        return true
    }

    let max_possible = (n as f64).sqrt().ceil() as uint;
    for i in iter::range(2, max_possible + 1) {
        if n % i == 0 {
            return false
        }
    }
    return true
}

fn prop_all_prime(n: uint) -> bool {
    let primes = sieve(n);
    debug!("{}: {}", n, primes);
    primes.iter().all(|&i| is_prime(i))
}

fn main() {
    quickcheck(prop_all_prime);
}
