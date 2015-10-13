use rand::{Rng, Open01, Closed01};
use rand::distributions::normal::StandardNormal;
use rand::distributions::exponential::Exp1;
use std::hash::{Hash, Hasher, SipHasher};
use std::marker;
use std::fmt::Debug;
use std::ops::{Range, RangeFrom, RangeTo, RangeFull};
use std::collections::{
    BTreeMap,
    BTreeSet,
    BinaryHeap,
    HashMap,
    HashSet,
    LinkedList,
    VecDeque,
};
use entropy_pool::{EntropyPool};
use shrink::{Shrinker, StdShrinker};

/// Whether a test has passed, failed or been discarded.
#[derive(Clone, Eq, Debug, PartialEq)]
pub enum Status { Pass, Fail, Discard }

/// Describes the status of a single instance of a test.
///
/// All testable things must be capable of producing a `TestResult`.
#[derive(Clone, Debug)]
pub struct TestResult<T> {
    pub status: Status,
    pub info: T,
    pub err: Option<String>,
}

/// `Testable` describes types (e.g., a function) whose values can be
/// tested.
///
/// Anything that can be tested must be capable of producing a `TestResult`
/// given a random number generator. This is trivial for types like `bool`,
/// which are just converted to either a passing or failing test result.
///
/// For functions, an implementation must generate random info
/// and potentially shrink those info if they produce a failure.
///
/// It's unlikely that you'll have to implement this trait yourself.
/// This comes with a caveat: currently, only functions with 4 parameters
/// or fewer (both `fn` and `||` types) satisfy `Testable`. If you have
/// functions to test with more than 4 parameters, please
/// [file a bug](https://github.com/BurntSushi/quickcheck/issues) and
/// I'll hopefully add it. (As of now, it would be very difficult to
/// add your own implementation outside of `quickcheck`, since the
/// functions that do shrinking are not public.)
///
///
pub trait Testable {
    type Info: Debug;
    fn result<G: Gen>(self, &mut G) -> TestResult<Self::Info>;
}

/// `Gen` wraps a `rand::Rng` with parameters to control the distribution of
/// random values and a test-running strategy.
///
/// A value with type satisfying the `Gen` trait can be constructed with the
/// `gen` function in this crate.
pub trait Gen: Rng {
    fn size(&mut self) -> &mut usize;

    fn run<T: Testable + Clone>(&mut self, t: T) -> TestResult<T::Info>;
}

/// StdGen is the default impelementation of `Gen`.
///
/// Values of type `StdGen` can be created with the `gen` function in this
/// crate. StdGen implements automatic strinking of test cases.
pub struct StdGen<R> {
    pool: EntropyPool<R>,
    size: usize,
}

/// Returns a `StdGen` with the given configuration using any random number
/// generator.
///
/// The `size` parameter controls the size of random values generated.
/// For example, it specifies the maximum length of a randomly generated
/// vector and also will specify the maximum magnitude of a randomly generated
/// number.
impl<R: Rng> StdGen<R> {
    pub fn new(rng: R, size: usize) -> StdGen<R> {
        StdGen { pool: EntropyPool::new(rng, 4 * size), size: size }
    }
}

impl<R: Rng> Rng for StdGen<R> {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 { self.pool.next_u32() }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 { self.pool.next_u64() }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) { self.pool.fill_bytes(dest) }
}

impl<R: Rng> Gen for StdGen<R> {
    fn size(&mut self) -> &mut usize {
        &mut self.size
    }

    fn run<T: Testable + Clone>(&mut self, t: T) -> TestResult<T::Info> {
        self.pool.randomize();
        self.pool.i = 0;
        type S = StdShrinker;

        fn shrink_once<R>(g: &mut StdGen<R>, restore_buffer: &mut Vec<u8>,
                          s: &mut S) -> bool {
            restore_buffer.clear();
            restore_buffer.extend(g.pool
                                   .v
                                   .iter()
                                   .rev()
                                   .skip_while(|&&w| w == 0));
            s.shrink(g.size, &mut g.pool.v)
        }
        let mut restore_buffer = Vec::new();
        let mut s = S::new(&self.pool.v[..]);
        let mut r = t.clone().result(self);
        if r.status != Status::Fail {
            return r;
        }
        let mut set = HashSet::new();
        loop {
            if !shrink_once(self, &mut restore_buffer, &mut s) {
                return r;
            }
            self.pool.i = 0;
            let r_next = t.clone().result(self);
            let j = {
                let v = &self.pool.v;
                let i = self.pool.i;
                (&v[0..i]).iter()
                          .enumerate()
                          .rev()
                          .skip_while(|&(_, &w)| w == 0)
                          .map(|u| u.0 + 1)
                          .next()
                          .unwrap_or(0)
            };
            let mut sip = SipHasher::new_with_keys(
                6152069331286739346,
                6442131328594060759
            );
            &self.pool.v[..j].hash(&mut sip);
            let hash = sip.finish();
            let is_new = if set.contains(&hash) {
                false
            } else {
                set.insert(hash);
                true
            };
            if is_new && r_next.status == Status::Fail {
                //println!("pool: {:?}", self.pool.v);
                r = r_next;
            } else {
                for (ptr, &w) in self.pool
                                     .v[..]
                                     .iter_mut()
                                     .zip(restore_buffer.iter().rev()) {
                    *ptr = w;
                }
            }
        }
    }
}

/// `Arbitrary` describes types whose values can be randomly generated
///
/// `Arbitrary` is different from the `std::Rand` trait
/// in that it draws from a `Gen`, allowing for shrinking and manipulating
/// the distribution of generated values.
///
/// Now that auto-shrinking is implemented, you should no longer implement
/// `shrink`.
pub trait Arbitrary {
    fn arbitrary<G: Gen>(g: &mut G) -> Self;
}

pub struct ArbitraryIterator<'a, G: 'a, T> {
    _marker: marker::PhantomData<T>,
    gen: &'a mut G,
    left: usize
}

impl <'a, G: Gen, T: Arbitrary>Iterator for ArbitraryIterator<'a, G, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.left == 0 {
            return None
        }
        self.left -= 1;
        Some(T::arbitrary(self.gen))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.left, Some(self.left))
    }
}

pub fn sized_arbitrary_iterator<'a, G: Gen, T: Arbitrary>(
    g: &'a mut G, size: usize
) -> ArbitraryIterator<'a, G, T> {
    ArbitraryIterator {
        _marker: marker::PhantomData::<T>,
        gen: g,
        left: size,
    }
}

pub fn arbitrary_iterator<'a, G: Gen, T: Arbitrary>(
    g: &'a mut G
) -> ArbitraryIterator<'a, G, T> {
    let size = { let s = *g.size(); g.gen_range(0, s + 1) };
    sized_arbitrary_iterator(g, size)
}

macro_rules! arbitrary_using_gen {
    ($ty:ty) => (
        impl Arbitrary for $ty {
            fn arbitrary<G: Gen>(g: &mut G) -> $ty {
                g.gen()
            }
        }
    );
}

arbitrary_using_gen!(());
arbitrary_using_gen!(bool);
arbitrary_using_gen!(char);
arbitrary_using_gen!(StandardNormal);
arbitrary_using_gen!(Exp1);
arbitrary_using_gen!(Open01<f32>);
arbitrary_using_gen!(Closed01<f32>);
arbitrary_using_gen!(Open01<f64>);
arbitrary_using_gen!(Closed01<f64>);

macro_rules! arbitrary_unsigned {
    ($ty:ty) => (
        impl Arbitrary for $ty {
            fn arbitrary<G: Gen>(g: &mut G) -> $ty {
                let s = *g.size() as $ty;
                g.gen_range(0 as $ty, s + (1 as $ty))
            }
        }
    );
}

arbitrary_unsigned!(usize);
arbitrary_unsigned!(u8);
arbitrary_unsigned!(u16);
arbitrary_unsigned!(u32);
arbitrary_unsigned!(u64);

macro_rules! arbitrary_signed {
    ($ty:ty) => (
        impl Arbitrary for $ty {
            fn arbitrary<G: Gen>(g: &mut G) -> $ty {
                let s = *g.size() as $ty;
                g.gen_range(-s, s + (1 as $ty))
            }
        }
    );
}

arbitrary_signed!(isize);
arbitrary_signed!(i8);
arbitrary_signed!(i16);
arbitrary_signed!(i32);
arbitrary_signed!(i64);
arbitrary_signed!(f32);
arbitrary_signed!(f64);

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
            Result::Ok(Arbitrary::arbitrary(g))
        } else {
            Result::Err(Arbitrary::arbitrary(g))
        }
    }
}

macro_rules! arbitrary_tuple {
    ($($tyvar:ident),* ) => {
        impl<$( $tyvar : Arbitrary ),*> Arbitrary for ( $( $tyvar ),* , ) {
            #[inline]
            fn arbitrary<G_: Gen>(g: &mut G_) -> ( $( $tyvar ),* , ) {
                ($({let x: $tyvar = Arbitrary::arbitrary(g); x}),*,)
            }
        }
    }
}

arbitrary_tuple!(A);
arbitrary_tuple!(A, B);
arbitrary_tuple!(A, B, C);
arbitrary_tuple!(A, B, C, D);
arbitrary_tuple!(A, B, C, D, E);
arbitrary_tuple!(A, B, C, D, E, F);
arbitrary_tuple!(A, B, C, D, E, F, G);
arbitrary_tuple!(A, B, C, D, E, F, G, H);
arbitrary_tuple!(A, B, C, D, E, F, G, H, I);
arbitrary_tuple!(A, B, C, D, E, F, G, H, I, J);
arbitrary_tuple!(A, B, C, D, E, F, G, H, I, J, K);
arbitrary_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

macro_rules! arbitrary_array {
    {$n:expr, $t:ident, $($ts:ident,)*} => {
        arbitrary_array!{($n - 1), $($ts,)*}

        impl<T: Arbitrary> Arbitrary for [T; $n] {
            #[inline]
            fn arbitrary<G: Gen>(g: &mut G) -> [T; $n] {
                [Arbitrary::arbitrary(g),
                 $({let x: $ts = Arbitrary::arbitrary(g); x}),*]
            }
        }
    };
    {$n:expr,} => {
        impl<T: Arbitrary> Arbitrary for [T; $n] {
            fn arbitrary<G: Gen>(_g: &mut G) -> [T; $n] { [] }
        }
    };
}

arbitrary_array!{32, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T,
                     T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T,}

impl<A: Arbitrary> Arbitrary for Vec<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Vec<A> {
        arbitrary_iterator(g).collect()
    }
}

impl<K: Arbitrary + Ord, V: Arbitrary> Arbitrary for BTreeMap<K, V> {
    fn arbitrary<G: Gen>(g: &mut G) -> BTreeMap<K, V> {
        arbitrary_iterator(g).collect()
    }
}

impl<K: Arbitrary + Eq + Hash, V: Arbitrary> Arbitrary for HashMap<K, V> {
    fn arbitrary<G: Gen>(g: &mut G) -> HashMap<K, V> {
        arbitrary_iterator(g).collect()
    }
}

impl<T: Arbitrary + Ord> Arbitrary for BTreeSet<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> BTreeSet<T> {
        arbitrary_iterator(g).collect()
    }
}

impl<T: Arbitrary + Ord> Arbitrary for BinaryHeap<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> BinaryHeap<T> {
        arbitrary_iterator(g).collect()
    }
}

impl<T: Arbitrary + Eq + Hash> Arbitrary for HashSet<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> HashSet<T> {
        arbitrary_iterator(g).collect()
    }
}

impl<T: Arbitrary> Arbitrary for LinkedList<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> LinkedList<T> {
        arbitrary_iterator(g).collect()
    }
}

impl<T: Arbitrary> Arbitrary for VecDeque<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> VecDeque<T> {
        arbitrary_iterator(g).collect()
    }
}

impl Arbitrary for String {
    fn arbitrary<G: Gen>(g: &mut G) -> String {
        arbitrary_iterator::<G, char>(g).collect()
    }
}

impl<T: Arbitrary + PartialOrd> Arbitrary for Range<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Range<T> {
        Arbitrary::arbitrary(g) .. Arbitrary::arbitrary(g)
    }
}

impl<T: Arbitrary + PartialOrd> Arbitrary for RangeFrom<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> RangeFrom<T> {
        Arbitrary::arbitrary(g) ..
    }
}

impl<T: Arbitrary + PartialOrd> Arbitrary for RangeTo<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> RangeTo<T> {
        .. Arbitrary::arbitrary(g)
    }
}

impl Arbitrary for RangeFull {
    fn arbitrary<G: Gen>(_: &mut G) -> RangeFull { .. }
}

