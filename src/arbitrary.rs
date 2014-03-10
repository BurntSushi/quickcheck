use std::rand;
use std::rand::{Rand, Rng};

pub trait Gen : Rng {
    fn size(&mut self) -> uint;
}

struct StdGen<R> {
    rng: R,
    size: uint,
}

impl<R: Rng> Rng for StdGen<R> {
    fn next_u32(&mut self) -> u32 { self.rng.next_u32() }
}

impl<R: Rng> Gen for StdGen<R> {
    fn size(&mut self) -> uint { self.size }
}

pub trait Arbitrary {
    fn arbitrary<G: Gen>(g: &mut G) -> Self;
}

impl Arbitrary for () {
    fn arbitrary<G: Gen>(_: &mut G) -> () { () }
}

impl Arbitrary for bool {
    fn arbitrary<G: Gen>(g: &mut G) -> bool {
        Rand::rand(g)
    }
}

pub fn arbitrary<A: Arbitrary>() -> A {
    Arbitrary::arbitrary(&mut gen(rand::task_rng(), 20))
}

pub fn gen<R: Rng>(rng: R, size: uint) -> StdGen<R> {
    StdGen{rng: rng, size: size}
}

#[cfg(test)]
mod tests {
    use super::arbitrary;

    #[test]
    fn unit() {
        assert_eq!(arbitrary::<()>(), ());
    }
}
