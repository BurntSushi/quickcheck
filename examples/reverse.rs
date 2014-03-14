extern crate quickcheck;

use quickcheck::quickcheck;

fn reverse<T: Clone>(xs: &[T]) -> ~[T] {
    let mut rev = ~[];
    for x in xs.iter() {
        rev.unshift(x.clone())
    }
    rev
}

fn main() {
    fn prop(xs: ~[int]) -> bool {
        xs == reverse(reverse(xs))
    }
    quickcheck(prop);
}
