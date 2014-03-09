use std::rand;
use std::rand::{Rand, Rng};

pub fn arbitrary<A: Arbitrary>() -> A {
    Arbitrary::arbitrary(&mut sized_rng(rand::rng(), 20))
}

pub fn sized_rng<R: Rng>(rng: R, size: uint) -> Gen<R> {
    Gen{rng: rng, size: size}
}

pub trait SizedRng : Rng {
    fn size(&mut self) -> uint;
}

struct Gen<R> {
    rng: R,
    size: uint,
}

impl<R: Rng> Rng for Gen<R> {
    fn next_u32(&mut self) -> u32 { self.rng.next_u32() }
}

impl<R: Rng> SizedRng for Gen<R> {
    fn size(&mut self) -> uint { self.size }
}

pub trait Arbitrary {
    fn arbitrary<R: SizedRng>(rng: &mut R) -> Self;
}

impl Arbitrary for () {
    fn arbitrary<R: SizedRng>(_: &mut R) -> () { () }
}

impl Arbitrary for bool {
    fn arbitrary<R: SizedRng>(rng: &mut R) -> bool {
        Rand::rand(rng)
    }
}

#[cfg(test)]
mod tests {
    use super::arbitrary;

    #[test]
    fn unit() {
        assert_eq!(arbitrary::<()>(), ());
    }
}
