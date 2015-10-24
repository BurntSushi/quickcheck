extern crate quickcheck;

use quickcheck::quickcheck;

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs {
        rev.insert(0, x.clone())
    }
    rev
}

fn main() {
    fn equality_after_applying_twice_vec(xs: Vec<isize>) -> bool {
        xs == reverse(&reverse(&xs))
    }
    quickcheck(equality_after_applying_twice_vec as fn(Vec<isize>) -> bool);

    fn equality_after_applying_twice_array(xs: [u8; 10]) -> bool {
        &xs[..] == &reverse(&reverse(&xs))[..]
    }
    quickcheck(equality_after_applying_twice_array as fn([u8; 10]) -> bool);
}
