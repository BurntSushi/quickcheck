#![feature(step_by)]

extern crate quickcheck;

use quickcheck::quickcheck;

fn sieve(n: usize) -> Vec<usize> {
    if n <= 1 {
        return vec!()
    }

    let mut marked: Vec<_> = (0..n+1).map(|_| false).collect();
    marked[0] = true;
    marked[1] = true;
    for p in 2..n {
        for i in (2*p..n).step_by(p) { // whoops!
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
    n != 0 && n != 1 && (2..).take_while(|i| i * i <= n).all(|i| n % i != 0)
}

fn prop_all_prime(n: usize) -> bool {
    sieve(n).iter().all(|&i| is_prime(i))
}

fn prop_prime_iff_in_the_sieve(n: usize) -> bool {
    (0..(n + 1)).filter(|&i| is_prime(i)).collect::<Vec<_>>() == sieve(n)
}

fn main() {
    quickcheck(prop_all_prime as fn(usize) -> bool);
    quickcheck(prop_prime_iff_in_the_sieve as fn(usize) -> bool);
}
