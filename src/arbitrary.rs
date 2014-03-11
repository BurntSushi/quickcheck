use std::rand::{Rng, TaskRng, task_rng};
use std::vec;

/// Gen wraps a `rand::Rng` with parameters to control the distribution of
/// random values.
pub trait Gen : Rng {
    fn size(&self) -> uint;
}

/// StdGen is the default implementation of `Gen`.
pub struct StdGen<R> {
    rng: R,
    size: uint,
}

impl<R: Rng> Rng for StdGen<R> {
    fn next_u32(&mut self) -> u32 { self.rng.next_u32() }
}

impl<R: Rng> Gen for StdGen<R> {
    fn size(&self) -> uint { self.size }
}

/// Arbitrary specifies how values of a particular type should be randomly
/// generated.
///
/// It is different from the `std::Rand` trait in that it uses a `Gen` to 
/// control the distribution of random values.
pub trait Arbitrary {
    fn arbitrary<G: Gen>(g: &mut G) -> Self;
}

impl<A: Arbitrary> Arbitrary for ~[A] {
    fn arbitrary<G: Gen>(g: &mut G) -> ~[A] {
        let size = { let s = g.size(); g.gen_range(0, s) };
        vec::from_fn(size, |_| Arbitrary::arbitrary(g))
    }
}

impl<A: Arbitrary> Arbitrary for Option<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Option<A> {
        if g.gen() {
            None
        } else {
            Some(Arbitrary::arbitrary(g))
        }
    }
}

impl<A: Arbitrary, B: Arbitrary> Arbitrary for Result<A, B> {
    fn arbitrary<G: Gen>(g: &mut G) -> Result<A, B> {
        if g.gen() {
            Ok(Arbitrary::arbitrary(g))
        } else {
            Err(Arbitrary::arbitrary(g))
        }
    }
}

impl<A: Arbitrary, B: Arbitrary> Arbitrary for (A, B) {
    fn arbitrary<G: Gen>(g: &mut G) -> (A, B) {
        return (Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
    }
}

impl<A: Arbitrary, B: Arbitrary, C: Arbitrary> Arbitrary for (A, B, C) {
    fn arbitrary<G: Gen>(g: &mut G) -> (A, B, C) {
        return (
            Arbitrary::arbitrary(g),
            Arbitrary::arbitrary(g),
            Arbitrary::arbitrary(g),
        )
    }
}

impl Arbitrary for () {
    fn arbitrary<G: Gen>(_: &mut G) -> () { () }
}

impl Arbitrary for bool {
    fn arbitrary<G: Gen>(g: &mut G) -> bool { g.gen() }
}

impl Arbitrary for ~str {
    fn arbitrary<G: Gen>(g: &mut G) -> ~str {
        let size = { let s = g.size(); g.gen_range(0, s) };
        g.gen_ascii_str(size)
    }
}

impl Arbitrary for char {
    fn arbitrary<G: Gen>(g: &mut G) -> char { g.gen() }
}

impl Arbitrary for int {
    fn arbitrary<G: Gen>(g: &mut G) -> int {
        let s = g.size(); g.gen_range(-(s as int), s as int)
    }
}

impl Arbitrary for i8 {
    fn arbitrary<G: Gen>(g: &mut G) -> i8 {
        let s = g.size(); g.gen_range(-(s as i8), s as i8)
    }
}

impl Arbitrary for i16 {
    fn arbitrary<G: Gen>(g: &mut G) -> i16 {
        let s = g.size(); g.gen_range(-(s as i16), s as i16)
    }
}

impl Arbitrary for i32 {
    fn arbitrary<G: Gen>(g: &mut G) -> i32 {
        let s = g.size(); g.gen_range(-(s as i32), s as i32)
    }
}

impl Arbitrary for i64 {
    fn arbitrary<G: Gen>(g: &mut G) -> i64 {
        let s = g.size(); g.gen_range(-(s as i64), s as i64)
    }
}

impl Arbitrary for uint {
    fn arbitrary<G: Gen>(g: &mut G) -> uint {
        let s = g.size(); g.gen_range(0, s)
    }
}

impl Arbitrary for u8 {
    fn arbitrary<G: Gen>(g: &mut G) -> u8 {
        let s = g.size(); g.gen_range(0, s as u8)
    }
}

impl Arbitrary for u16 {
    fn arbitrary<G: Gen>(g: &mut G) -> u16 {
        let s = g.size(); g.gen_range(0, s as u16)
    }
}

impl Arbitrary for u32 {
    fn arbitrary<G: Gen>(g: &mut G) -> u32 {
        let s = g.size(); g.gen_range(0, s as u32)
    }
}

impl Arbitrary for u64 {
    fn arbitrary<G: Gen>(g: &mut G) -> u64 {
        let s = g.size(); g.gen_range(0, s as u64)
    }
}

/// Returns a random value according to `A`'s `Arbitrary` implementation using
/// the default `Gen`.
pub fn arbitrary<A: Arbitrary>() -> A {
    Arbitrary::arbitrary(&mut default_gen())
}

/// Returns a default implementation for `Gen` using a task-local random number
/// generator.
pub fn default_gen() -> StdGen<TaskRng> {
    gen(task_rng(), 20)
}

/// Returns a `Gen` with the given configuration.
///
/// The `size` parameter controls the size of random values generated.
/// For example, it specifies the maximum length of a randomly generator vector
/// and also will specify the maximum magnitude of a randomly generated number.
pub fn gen<R: Rng>(rng: R, size: uint) -> StdGen<R> {
    StdGen{rng: rng, size: size}
}

#[cfg(test)]
mod test {
    use std::iter;
    use std::rand;

    #[test]
    fn unit() {
        assert_eq!(arby::<()>(), ());
    }

    #[test]
    fn int() {
        rep(|| { let n: int = arby(); assert!(n >= -5 && n <= 5); } );
    }

    #[test]
    fn uint() {
        rep(|| { let n: uint = arby(); assert!(n <= 5); } );
    }

    fn arby<A: super::Arbitrary>() -> A {
        super::Arbitrary::arbitrary(&mut gen())
    }

    fn rep(f: ||) {
        for _ in iter::range(0, 100) {
            f()
        }
    }
}
