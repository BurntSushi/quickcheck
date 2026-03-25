use quickcheck::{quickcheck, TestFailure};

fn sieve(n: usize) -> Vec<usize> {
    if n <= 1 {
        return vec![];
    }

    let mut marked = vec![false; n + 1];
    marked[0] = true;
    marked[1] = true;
    marked[2] = true;
    for p in 2..n {
        for i in (2 * p..n).filter(|&n| n % p == 0) {
            marked[i] = true;
        }
    }
    marked
        .iter()
        .enumerate()
        .filter_map(|(i, &m)| if m { None } else { Some(i) })
        .collect()
}

fn is_prime(n: usize) -> bool {
    n != 0 && n != 1 && (2..).take_while(|i| i * i <= n).all(|i| n % i != 0)
}

// This function demonstrates how to use `TestFailure` to factor test logic into
// a function with an ergonomic API. In this case, we want to check that all
// numbers returned by `sieve` are prime, and we want to return a specific error
// message indicating which non-prime number was found if the test fails.
fn check_prime(n: usize) -> Result<(), TestFailure> {
    if is_prime(n) {
        Ok(())
    } else {
        Err(TestFailure::error(format!("{} is not prime", n)))
    }
}

fn main() {
    fn prop_all_prime(n: usize) -> Result<(), TestFailure> {
        for i in sieve(n) {
            // Return early in case of failure.
            check_prime(i)?;
        }
        Ok(())
    }

    fn prop_prime_iff_in_the_sieve(n: usize) -> bool {
        sieve(n) == (0..=n).filter(|&i| is_prime(i)).collect::<Vec<_>>()
    }

    quickcheck(prop_all_prime as fn(usize) -> _);
    quickcheck(prop_prime_iff_in_the_sieve as fn(usize) -> bool);
}
